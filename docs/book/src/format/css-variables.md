# CSS Variables

Palettes support CSS custom properties (variables) for dynamic theming and reusable color definitions. This follows standard CSS variable syntax with full support for fallback values and nested references.

## Basic Syntax

Define variables with the `--` prefix and reference them with `var()`:

```json
{
  "type": "palette",
  "name": "themed",
  "colors": {
    "--primary": "#4169E1",
    "--accent": "#FFD700",
    "{_}": "transparent",
    "{main}": "var(--primary)",
    "{highlight}": "var(--accent)"
  }
}
```

## Variable Definition

<!-- DEMOS format/css/variables#definition -->
**Tests CSS custom property definition syntax (--name: value)**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "theme_colors", "colors": {"{_}": "#00000000", "{primary}": "#FF0000", "{secondary}": "#00FF00"}}
{"type": "sprite", "name": "theme_example", "palette": "theme_colors", "size": [2, 2], "regions": {"primary": {"rect": [0, 0, 1, 2]}, "secondary": {"rect": [1, 0, 1, 2]}}}
```

</div>

<div class="demo-container" data-demo="definition">
</div>
<!-- /DEMOS -->

Variables are palette entries with keys starting with `--`:

| Syntax | Description |
|--------|-------------|
| `"--name": "value"` | Define a variable |
| `"--kebab-case": "#hex"` | Convention: use kebab-case |

Variables are resolved in a two-pass process:
1. **Collection pass**: All `--name` entries are collected
2. **Resolution pass**: `var()` references are expanded

This allows forward references - a color can use `var(--name)` even if `--name` is defined later in the palette.

## Variable References

<!-- DEMOS format/css/variables#resolution -->
**Tests var() reference resolution**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "var_resolution", "colors": {"{_}": "#00000000", "{a}": "#FF0000", "{b}": "#00FF00"}}
{"type": "sprite", "name": "resolved_colors", "palette": "var_resolution", "size": [2, 2], "regions": {"a": {"rect": [0, 0, 1, 2]}, "b": {"rect": [1, 0, 1, 2]}}}
```

</div>

<div class="demo-container" data-demo="resolution">
</div>
<!-- /DEMOS -->

Reference variables with `var(--name)` or `var(--name, fallback)`:

| Syntax | Description |
|--------|-------------|
| `var(--name)` | Simple reference |
| `var(--name, fallback)` | Reference with fallback value |
| `var(name)` | Also works (-- prefix optional) |

### Fallback Values

<!-- DEMOS format/css/variables#fallbacks -->
**Tests var(--name, fallback) syntax**

<div class="demo-source">

```jsonl
{"type": "palette", "name": "simple_fallback", "colors": {"{_}": "#00000000", "{fb}": "#FF0000"}}
{"type": "palette", "name": "nested_fallback", "colors": {"{_}": "#00000000", "{nf}": "#00FF00"}}
{"type": "palette", "name": "color_mix_fallback", "colors": {"{_}": "#00000000", "{mf}": "#0000FF"}}
{"type": "sprite", "name": "fallback_demo", "palette": "simple_fallback", "size": [2, 2], "regions": {"fb": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "nested_fallback_result", "palette": "nested_fallback", "size": [2, 2], "regions": {"nf": {"rect": [0, 0, 2, 2]}}}
{"type": "sprite", "name": "mix_fallback_result", "palette": "color_mix_fallback", "size": [2, 2], "regions": {"mf": {"rect": [0, 0, 2, 2]}}}
```

</div>

<div class="demo-container" data-demo="fallbacks">
</div>
<!-- /DEMOS -->

Fallbacks are used when a variable is undefined:

```json
{
  "colors": {
    "--primary": "#FF0000",
    "{main}": "var(--primary)",
    "{alt}": "var(--secondary, #00FF00)"
  }
}
```

If `--secondary` is not defined, `{alt}` uses the fallback `#00FF00`.

### Nested References

Fallbacks can contain `var()` references:

```json
{
  "colors": {
    "--base": "#FF0000",
    "{color}": "var(--override, var(--base))"
  }
}
```

This resolves `--override` if defined, otherwise falls back to `--base`.

## Use Cases

### Theming

Define a theme with variables, then reference them:

```json
{"type": "palette", "name": "dark_theme", "colors": {
  "--bg": "#1A1A2E",
  "--fg": "#EAEAEA",
  "--accent": "#E94560",
  "{_}": "transparent",
  "{background}": "var(--bg)",
  "{text}": "var(--fg)",
  "{highlight}": "var(--accent)"
}}
```

### Color Components

Variables can contain partial values for CSS color functions:

```json
{
  "colors": {
    "--r": "255",
    "--g": "128",
    "--b": "0",
    "{orange}": "rgb(var(--r), var(--g), var(--b))"
  }
}
```

Or HSL components:

```json
{
  "colors": {
    "--hue": "240",
    "--sat": "100%",
    "--light": "50%",
    "{blue}": "hsl(var(--hue), var(--sat), var(--light))"
  }
}
```

### Optional Overrides

Use fallbacks for optional customization:

```json
{
  "colors": {
    "--primary": "#4169E1",
    "{main}": "var(--primary)",
    "{alt}": "var(--alt-color, var(--primary))"
  }
}
```

If `--alt-color` is defined, it's used; otherwise falls back to `--primary`.

## Error Handling

### Lenient Mode (Default)

In lenient mode, errors produce warnings and use magenta (`#FF00FF`) as a fallback:

- Undefined variable without fallback: magenta
- Circular reference detected: magenta

```bash
$ pxl render sprite.jsonl
Warning: variable error for '{color}': undefined variable '--missing' with no fallback
```

### Strict Mode

In strict mode (`--strict`), variable errors cause the command to fail:

```bash
$ pxl render sprite.jsonl --strict
Error: variable error for '{color}': undefined variable '--missing' with no fallback
```

### Circular References

Variables that reference each other create a circular dependency:

```json
{
  "colors": {
    "--a": "var(--b)",
    "--b": "var(--a)",
    "{color}": "var(--a)"
  }
}
```

This produces an error: `circular dependency: --a -> --b -> --a`

## Best Practices

1. **Use descriptive names**: `--primary-bg` over `--bg1`
2. **Group related variables**: Keep theme colors together
3. **Provide fallbacks for optional variables**: Enables safe customization
4. **Use kebab-case**: Matches CSS convention (`--my-color`)
5. **Avoid deep nesting**: Keep reference chains shallow for readability

## Compatibility

CSS variables follow the CSS Custom Properties specification with these notes:

- Variable names are normalized (with or without `--` prefix)
- Whitespace in `var()` is trimmed
- All CSS color formats work in variables (hex, rgb, hsl, named colors)
- Resolution depth is limited to prevent stack overflow (100 levels)

## Example

Complete example with theming:

```jsonl
{"type": "palette", "name": "dracula", "colors": {
  "--bg": "#282A36",
  "--fg": "#F8F8F2",
  "--comment": "#6272A4",
  "--cyan": "#8BE9FD",
  "--green": "#50FA7B",
  "--orange": "#FFB86C",
  "--pink": "#FF79C6",
  "--purple": "#BD93F9",
  "--red": "#FF5555",
  "--yellow": "#F1FA8C",
  "{_}": "transparent",
  "{bg}": "var(--bg)",
  "{outline}": "var(--comment)",
  "{skin}": "var(--orange)",
  "{hair}": "var(--purple)",
  "{eyes}": "var(--cyan)",
  "{shirt}": "var(--pink)"
}}
{"type": "sprite", "name": "character", "size": [5, 5], "palette": "dracula", "regions": {
  "hair": {"union": [{"rect": [1, 0, 3, 1]}, {"points": [[0, 1], [4, 1]]}], "z": 0},
  "skin": {"union": [{"rect": [1, 1, 3, 1]}, {"points": [[2, 2]]}, {"rect": [1, 3, 3, 1]}], "z": 1},
  "outline": {"points": [[0, 2], [4, 2]], "z": 0},
  "eyes": {"points": [[1, 2], [3, 2]], "z": 2},
  "shirt": {"rect": [1, 4, 3, 1], "z": 0}
}}
```

This creates a character sprite using Dracula theme colors, with all colors defined as variables for easy theme switching.
