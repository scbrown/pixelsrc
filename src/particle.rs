//! Particle system simulation and rendering engine.
//!
//! Implements particle spawning, physics simulation (velocity, gravity),
//! lifetime management, fade-out, and per-frame rendering into animation output.
//!
//! # Architecture
//!
//! The particle engine works in a pre-computed frame generation model:
//!
//! 1. Create a [`ParticleEngine`] from a [`Particle`] definition and sprite image
//! 2. Call [`ParticleEngine::generate_frames`] to simulate and render all frames
//! 3. Use the resulting `Vec<RgbaImage>` with GIF output or composition layers
//!
//! # Example
//!
//! ```ignore
//! use pixelsrc::particle::ParticleEngine;
//! use pixelsrc::models::Particle;
//! use image::RgbaImage;
//!
//! let particle_def = Particle { /* ... */ };
//! let sprite_image = RgbaImage::new(4, 4);
//! let engine = ParticleEngine::new(&particle_def, &sprite_image);
//! let frames = engine.generate_frames(60, [64, 64], [32, 32]);
//! ```

use image::{Rgba, RgbaImage};

use crate::models::Particle;

/// A live particle instance during simulation.
#[derive(Debug, Clone)]
#[allow(dead_code)] // rotation stored for future sprite rotation support
struct LiveParticle {
    /// Current X position (sub-pixel precision)
    x: f64,
    /// Current Y position (sub-pixel precision)
    y: f64,
    /// X velocity (pixels per frame)
    vx: f64,
    /// Y velocity (pixels per frame)
    vy: f64,
    /// Current age in frames
    age: u32,
    /// Maximum lifetime in frames
    lifetime: u32,
    /// Rotation in degrees (if applicable)
    rotation: f64,
}

impl LiveParticle {
    /// Returns the normalized age (0.0 = just born, 1.0 = about to die).
    fn normalized_age(&self) -> f64 {
        if self.lifetime == 0 {
            1.0
        } else {
            self.age as f64 / self.lifetime as f64
        }
    }

    /// Returns whether this particle has exceeded its lifetime.
    fn is_dead(&self) -> bool {
        self.age >= self.lifetime
    }
}

/// A simple deterministic PRNG (xorshift64) for reproducible particle effects.
#[derive(Debug, Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        // Ensure non-zero state
        Self { state: if seed == 0 { 0x12345678_9ABCDEF0 } else { seed } }
    }

    /// Generate next u64 value.
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generate a random f64 in [0.0, 1.0).
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Generate a random f64 in [min, max].
    fn range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }

    /// Generate a random u32 in [min, max].
    fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % (max - min + 1) as u64) as u32
    }
}

/// Particle system simulation and rendering engine.
///
/// Takes a particle definition and its sprite image, then simulates and
/// renders particle effects frame-by-frame.
pub struct ParticleEngine<'a> {
    /// The particle system definition
    def: &'a Particle,
    /// The sprite image used for each particle
    sprite: &'a RgbaImage,
    /// Random number generator
    rng: Rng,
}

impl<'a> ParticleEngine<'a> {
    /// Create a new particle engine from a definition and sprite image.
    pub fn new(def: &'a Particle, sprite: &'a RgbaImage) -> Self {
        let seed = def.emitter.seed.unwrap_or(42);
        Self { def, sprite, rng: Rng::new(seed) }
    }

    /// Generate all frames for the particle system.
    ///
    /// # Arguments
    ///
    /// * `frame_count` - Number of frames to generate
    /// * `canvas_size` - Output canvas size `[width, height]`
    /// * `origin` - Emitter origin position `[x, y]` on the canvas
    ///
    /// # Returns
    ///
    /// A vector of RGBA images, one per frame.
    pub fn generate_frames(
        &mut self,
        frame_count: u32,
        canvas_size: [u32; 2],
        origin: [i32; 2],
    ) -> Vec<RgbaImage> {
        let mut particles: Vec<LiveParticle> = Vec::new();
        let mut frames = Vec::with_capacity(frame_count as usize);

        // Fractional accumulator for sub-frame emission rates
        let mut emit_accumulator: f64 = 0.0;

        for _frame in 0..frame_count {
            // 1. Spawn new particles
            emit_accumulator += self.def.emitter.rate;
            let to_spawn = emit_accumulator.floor() as u32;
            emit_accumulator -= to_spawn as f64;

            for _ in 0..to_spawn {
                particles.push(self.spawn_particle(origin));
            }

            // 2. Render current state
            let canvas = self.render_frame(&particles, canvas_size);
            frames.push(canvas);

            // 3. Update physics (velocity + gravity)
            let gravity = self.def.emitter.gravity.unwrap_or(0.0);
            for p in &mut particles {
                p.x += p.vx;
                p.y += p.vy;
                p.vy += gravity;
                p.age += 1;
            }

            // 4. Remove dead particles
            particles.retain(|p| !p.is_dead());
        }

        frames
    }

    /// Spawn a single particle at the emitter origin with randomized properties.
    fn spawn_particle(&mut self, origin: [i32; 2]) -> LiveParticle {
        let emitter = &self.def.emitter;

        // Velocity
        let (vx, vy) = if let Some(ref vel) = emitter.velocity {
            (self.rng.range(vel.x[0], vel.x[1]), self.rng.range(vel.y[0], vel.y[1]))
        } else {
            (0.0, 0.0)
        };

        // Lifetime
        let lifetime = self.rng.range_u32(emitter.lifetime[0], emitter.lifetime[1]);

        // Rotation
        let rotation =
            if let Some(ref rot) = emitter.rotation { self.rng.range(rot[0], rot[1]) } else { 0.0 };

        LiveParticle {
            x: origin[0] as f64,
            y: origin[1] as f64,
            vx,
            vy,
            age: 0,
            lifetime,
            rotation,
        }
    }

    /// Render all live particles onto a canvas for one frame.
    fn render_frame(&self, particles: &[LiveParticle], canvas_size: [u32; 2]) -> RgbaImage {
        let mut canvas = RgbaImage::new(canvas_size[0], canvas_size[1]);
        let fade = self.def.emitter.fade.unwrap_or(false);

        for p in particles {
            let opacity = if fade { 1.0 - p.normalized_age() } else { 1.0 };

            if opacity <= 0.0 {
                continue;
            }

            self.blit_particle(&mut canvas, p, opacity);
        }

        canvas
    }

    /// Blit a single particle's sprite onto the canvas with alpha and fade.
    fn blit_particle(&self, canvas: &mut RgbaImage, p: &LiveParticle, opacity: f64) {
        let sw = self.sprite.width() as i32;
        let sh = self.sprite.height() as i32;
        let cw = canvas.width() as i32;
        let ch = canvas.height() as i32;

        // Center the sprite on the particle position
        let px = p.x.round() as i32 - sw / 2;
        let py = p.y.round() as i32 - sh / 2;

        for sy in 0..sh {
            let dy = py + sy;
            if dy < 0 || dy >= ch {
                continue;
            }

            for sx in 0..sw {
                let dx = px + sx;
                if dx < 0 || dx >= cw {
                    continue;
                }

                let src = *self.sprite.get_pixel(sx as u32, sy as u32);
                if src[3] == 0 {
                    continue;
                }

                // Apply fade opacity to source alpha
                let src_alpha = (src[3] as f64 / 255.0) * opacity;
                if src_alpha <= 0.0 {
                    continue;
                }

                let dst = *canvas.get_pixel(dx as u32, dy as u32);
                let blended = alpha_blend(&src, &dst, src_alpha);
                canvas.put_pixel(dx as u32, dy as u32, blended);
            }
        }
    }
}

/// Alpha-blend source over destination with modified source alpha.
fn alpha_blend(src: &Rgba<u8>, dst: &Rgba<u8>, src_alpha: f64) -> Rgba<u8> {
    let sa = src_alpha;
    let da = dst[3] as f64 / 255.0;

    // Standard "source over" compositing
    let out_a = sa + da * (1.0 - sa);
    if out_a <= 0.0 {
        return Rgba([0, 0, 0, 0]);
    }

    let blend = |s: u8, d: u8| -> u8 {
        let sf = s as f64 / 255.0;
        let df = d as f64 / 255.0;
        let out = (sf * sa + df * da * (1.0 - sa)) / out_a;
        (out * 255.0).round().clamp(0.0, 255.0) as u8
    };

    Rgba([
        blend(src[0], dst[0]),
        blend(src[1], dst[1]),
        blend(src[2], dst[2]),
        (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ParticleEmitter, VelocityRange};

    /// Helper to create a 2x2 white pixel sprite.
    fn test_sprite() -> RgbaImage {
        RgbaImage::from_pixel(2, 2, Rgba([255, 255, 255, 255]))
    }

    /// Helper to create a basic particle definition.
    fn test_particle() -> Particle {
        Particle {
            name: "test".to_string(),
            sprite: "spark".to_string(),
            emitter: ParticleEmitter {
                rate: 1.0,
                lifetime: [5, 10],
                velocity: Some(VelocityRange { x: [-1.0, 1.0], y: [-2.0, 0.0] }),
                gravity: Some(0.1),
                fade: Some(true),
                rotation: None,
                seed: Some(123),
            },
        }
    }

    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_rng_range() {
        let mut rng = Rng::new(99);
        for _ in 0..1000 {
            let v = rng.range(-5.0, 5.0);
            assert!((-5.0..=5.0).contains(&v), "Value {} out of range", v);
        }
    }

    #[test]
    fn test_rng_range_u32() {
        let mut rng = Rng::new(99);
        for _ in 0..1000 {
            let v = rng.range_u32(3, 10);
            assert!((3..=10).contains(&v), "Value {} out of range", v);
        }
    }

    #[test]
    fn test_live_particle_normalized_age() {
        let p =
            LiveParticle { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, age: 5, lifetime: 10, rotation: 0.0 };
        assert!((p.normalized_age() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_live_particle_is_dead() {
        let mut p =
            LiveParticle { x: 0.0, y: 0.0, vx: 0.0, vy: 0.0, age: 9, lifetime: 10, rotation: 0.0 };
        assert!(!p.is_dead());
        p.age = 10;
        assert!(p.is_dead());
    }

    #[test]
    fn test_generate_frames_count() {
        let def = test_particle();
        let sprite = test_sprite();
        let mut engine = ParticleEngine::new(&def, &sprite);

        let frames = engine.generate_frames(20, [32, 32], [16, 16]);
        assert_eq!(frames.len(), 20);
    }

    #[test]
    fn test_generate_frames_canvas_size() {
        let def = test_particle();
        let sprite = test_sprite();
        let mut engine = ParticleEngine::new(&def, &sprite);

        let frames = engine.generate_frames(5, [64, 48], [32, 24]);
        for frame in &frames {
            assert_eq!(frame.width(), 64);
            assert_eq!(frame.height(), 48);
        }
    }

    #[test]
    fn test_particles_appear_at_origin() {
        let def = Particle {
            name: "static".to_string(),
            sprite: "dot".to_string(),
            emitter: ParticleEmitter {
                rate: 1.0,
                lifetime: [100, 100],
                velocity: None,
                gravity: None,
                fade: None,
                rotation: None,
                seed: Some(1),
            },
        };
        let sprite = RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255]));
        let mut engine = ParticleEngine::new(&def, &sprite);

        let frames = engine.generate_frames(1, [16, 16], [8, 8]);
        // The particle at (8,8) with 1x1 sprite should color pixel (8,8)
        let pixel = frames[0].get_pixel(8, 8);
        assert_eq!(pixel[0], 255, "Red channel should be 255");
        assert_eq!(pixel[3], 255, "Alpha should be fully opaque");
    }

    #[test]
    fn test_gravity_moves_particles_down() {
        // Use a single particle by emitting once then stopping
        let def = Particle {
            name: "falling".to_string(),
            sprite: "dot".to_string(),
            emitter: ParticleEmitter {
                rate: 1.0,
                lifetime: [100, 100],
                velocity: None,
                gravity: Some(1.0),
                fade: None,
                rotation: None,
                seed: Some(1),
            },
        };
        let sprite = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255]));
        let mut engine = ParticleEngine::new(&def, &sprite);

        let frames = engine.generate_frames(6, [32, 32], [16, 4]);
        // Frame 0: spawn at (16,4), render at y=4, update: y stays 4, vy becomes 1
        let has_pixel_at_origin = frames[0].get_pixel(16, 4)[3] > 0;
        assert!(has_pixel_at_origin, "First frame should have particle at origin");

        // After several frames, the first particle should be below origin
        // Frame 0: render y=4, then y=4+0=4, vy=0+1=1
        // Frame 1: render y=4, then y=4+1=5, vy=1+1=2
        // Frame 2: render y=5, then y=5+2=7, vy=2+1=3
        // Frame 3: render y=7, then y=7+3=10, vy=3+1=4
        // Frame 4: render y=10, then y=10+4=14, vy=4+1=5
        // Frame 5: render y=14
        // Check that a pixel exists below origin by frame 5
        let has_pixel_far_below = frames[5].get_pixel(16, 14)[3] > 0;
        assert!(has_pixel_far_below, "Gravity should move particles downward");
    }

    #[test]
    fn test_fade_reduces_opacity() {
        // Give particles velocity so they spread out and don't stack
        let def = Particle {
            name: "fading".to_string(),
            sprite: "dot".to_string(),
            emitter: ParticleEmitter {
                rate: 1.0,
                lifetime: [20, 20],
                velocity: Some(VelocityRange { x: [3.0, 3.0], y: [0.0, 0.0] }),
                gravity: None,
                fade: Some(true),
                rotation: None,
                seed: Some(1),
            },
        };
        let sprite = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255]));
        let mut engine = ParticleEngine::new(&def, &sprite);

        let frames = engine.generate_frames(8, [64, 8], [4, 4]);

        // Frame 0: particle A spawns at x=4, rendered at x=4, then moves to x=7
        // Frame 1: particle B spawns at x=4, A rendered at x=7 (age 1), then A→x=10
        // ...
        // Frame 5: A rendered at x=4+3*5=19 (age 5, normalized_age=5/20=0.25, opacity=0.75)
        // Check that the first particle's alpha decreases over time by reading
        // its position in later frames.
        // Frame 0: A at x=4 with age=0 → full opacity
        let alpha_early = frames[0].get_pixel(4, 4)[3];
        // Frame 7: A at x=4+3*7=25 with age=7, normalized_age=7/20=0.35, opacity=0.65
        let alpha_late = frames[7].get_pixel(25, 4)[3];

        assert!(alpha_early > 0, "Should have pixel at origin on frame 0");
        assert!(alpha_late > 0, "Should have pixel at moved position");
        assert!(
            alpha_early > alpha_late,
            "Fade should reduce alpha: {} vs {}",
            alpha_early,
            alpha_late
        );
    }

    #[test]
    fn test_deterministic_with_seed() {
        let def = test_particle();
        let sprite = test_sprite();

        let mut engine1 = ParticleEngine::new(&def, &sprite);
        let frames1 = engine1.generate_frames(10, [32, 32], [16, 16]);

        let mut engine2 = ParticleEngine::new(&def, &sprite);
        let frames2 = engine2.generate_frames(10, [32, 32], [16, 16]);

        for (i, (f1, f2)) in frames1.iter().zip(frames2.iter()).enumerate() {
            assert_eq!(f1, f2, "Frame {} should be identical with same seed", i);
        }
    }

    #[test]
    fn test_different_seeds_differ() {
        let sprite = test_sprite();
        let def1 = Particle {
            name: "a".to_string(),
            sprite: "s".to_string(),
            emitter: ParticleEmitter {
                rate: 5.0,
                lifetime: [5, 15],
                velocity: Some(VelocityRange { x: [-3.0, 3.0], y: [-3.0, 3.0] }),
                gravity: None,
                fade: None,
                rotation: None,
                seed: Some(100),
            },
        };
        let def2 = Particle {
            emitter: ParticleEmitter { seed: Some(200), ..def1.emitter.clone() },
            ..def1.clone()
        };

        let mut engine1 = ParticleEngine::new(&def1, &sprite);
        let mut engine2 = ParticleEngine::new(&def2, &sprite);

        let frames1 = engine1.generate_frames(10, [32, 32], [16, 16]);
        let frames2 = engine2.generate_frames(10, [32, 32], [16, 16]);

        // At least some frames should differ
        let any_different = frames1.iter().zip(frames2.iter()).any(|(f1, f2)| f1 != f2);
        assert!(any_different, "Different seeds should produce different output");
    }

    #[test]
    fn test_emission_rate_fractional() {
        // rate=0.5 means one particle every 2 frames
        let def = Particle {
            name: "slow".to_string(),
            sprite: "dot".to_string(),
            emitter: ParticleEmitter {
                rate: 0.5,
                lifetime: [100, 100],
                velocity: None,
                gravity: None,
                fade: None,
                rotation: None,
                seed: Some(1),
            },
        };
        let sprite = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 128]));
        let mut engine = ParticleEngine::new(&def, &sprite);
        let frames = engine.generate_frames(4, [8, 8], [4, 4]);

        // Frame 0: accumulator 0.5, floor=0 -> no spawn
        // Frame 1: accumulator 1.0, floor=1 -> 1 spawn
        // Frame 2: accumulator 0.5, floor=0 -> no spawn
        // Frame 3: accumulator 1.0, floor=1 -> 1 spawn

        // Frame 0: empty
        assert_eq!(frames[0].get_pixel(4, 4)[3], 0, "Frame 0 should have no particles");
        // Frame 1: one particle spawned
        assert!(frames[1].get_pixel(4, 4)[3] > 0, "Frame 1 should have a particle");
    }

    #[test]
    fn test_particles_removed_after_lifetime() {
        let def = Particle {
            name: "short".to_string(),
            sprite: "dot".to_string(),
            emitter: ParticleEmitter {
                rate: 1.0,
                lifetime: [3, 3],
                velocity: None,
                gravity: None,
                fade: None,
                rotation: None,
                seed: Some(1),
            },
        };
        let sprite = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255]));
        let mut engine = ParticleEngine::new(&def, &sprite);

        // Generate enough frames that early particles should die
        let frames = engine.generate_frames(10, [8, 8], [4, 4]);
        // All frames should render without panic
        assert_eq!(frames.len(), 10);
    }

    #[test]
    fn test_alpha_blend() {
        // Fully opaque source over transparent destination
        let src = Rgba([255, 0, 0, 255]);
        let dst = Rgba([0, 0, 0, 0]);
        let result = super::alpha_blend(&src, &dst, 1.0);
        assert_eq!(result, Rgba([255, 0, 0, 255]));

        // Half-opacity source over opaque destination
        let src = Rgba([255, 0, 0, 255]);
        let dst = Rgba([0, 0, 255, 255]);
        let result = super::alpha_blend(&src, &dst, 0.5);
        // 50% red over blue = purple-ish
        assert!(result[0] > 100); // Some red
        assert!(result[2] > 100); // Some blue
        assert_eq!(result[3], 255); // Full alpha
    }

    #[test]
    fn test_zero_frame_count() {
        let def = test_particle();
        let sprite = test_sprite();
        let mut engine = ParticleEngine::new(&def, &sprite);

        let frames = engine.generate_frames(0, [32, 32], [16, 16]);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_velocity_moves_particles() {
        let def = Particle {
            name: "moving".to_string(),
            sprite: "dot".to_string(),
            emitter: ParticleEmitter {
                rate: 1.0,
                lifetime: [100, 100],
                velocity: Some(VelocityRange { x: [2.0, 2.0], y: [0.0, 0.0] }),
                gravity: None,
                fade: None,
                rotation: None,
                seed: Some(1),
            },
        };
        let sprite = RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255]));
        let mut engine = ParticleEngine::new(&def, &sprite);

        let frames = engine.generate_frames(5, [32, 32], [4, 4]);
        // Frame 0: particle at x=4
        assert!(frames[0].get_pixel(4, 4)[3] > 0);
        // Frame 3: particle rendered at frame 2 position (x=4), then updated
        // Frame 0: spawn at (4,4), render, then move to (6,4)
        // Frame 1: spawn new at (4,4), render old at (6,4) + new at (4,4), then old->(8,4)
        // Frame 2: render old at (8,4), then move to (10,4)
        // Frame 3: render old at (10,4)
        assert!(frames[3].get_pixel(10, 4)[3] > 0, "Particle should have moved right");
    }
}
