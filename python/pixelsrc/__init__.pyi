"""Type stubs for the pixelsrc package."""

from __future__ import annotations

from typing import Any, TYPE_CHECKING

if TYPE_CHECKING:
    import numpy as np
    from PIL import Image

__version__: str

# -- Stateless rendering functions --

def render_to_png(pxl: str) -> bytes:
    """Render the first sprite in a PXL string to PNG bytes.

    Parses the PXL/JSONL input, resolves palettes, and renders the first
    sprite found as a PNG image.

    Args:
        pxl: PXL or JSONL format string containing sprite and palette definitions.

    Returns:
        PNG image data as bytes, or empty bytes if no sprites are found.
    """
    ...

def render_to_rgba(pxl: str) -> RenderResult:
    """Render the first sprite in a PXL string to RGBA pixel data.

    Parses the PXL/JSONL input, resolves palettes, and renders the first
    sprite found as raw RGBA pixels.

    Args:
        pxl: PXL or JSONL format string containing sprite and palette definitions.

    Returns:
        A ``RenderResult`` with width, height, raw RGBA pixels, and any warnings.
    """
    ...

# -- Parsing and listing functions --

def parse(pxl: str) -> list[dict[str, Any]]:
    """Parse a PXL string and return a list of parsed objects as dicts.

    Each dict has a ``"type"`` key (``"palette"``, ``"sprite"``, ``"variant"``,
    etc.) plus all the fields for that object type. Parse warnings are appended
    as dicts with ``"warning"`` and ``"line"`` keys.

    Args:
        pxl: PXL or JSONL format string to parse.

    Returns:
        List of dicts representing parsed objects and any parse warnings.
    """
    ...

def list_sprites(pxl: str) -> list[str]:
    """Return a list of sprite names found in a PXL string.

    Args:
        pxl: PXL or JSONL format string to inspect.

    Returns:
        List of sprite names in definition order.
    """
    ...

def list_palettes(pxl: str) -> list[str]:
    """Return a list of palette names found in a PXL string.

    Args:
        pxl: PXL or JSONL format string to inspect.

    Returns:
        List of palette names in definition order.
    """
    ...

def format_pxl(pxl: str) -> str:
    """Format a PXL string for readability.

    Parses the input and reformats each object: sprites get grid arrays
    expanded to one row per line, compositions get layer maps expanded,
    and palettes/animations/variants are kept as single-line JSON.

    Args:
        pxl: PXL or JSONL format string to format.

    Returns:
        The formatted string.

    Raises:
        ValueError: If the input cannot be parsed.
    """
    ...

# -- Validation functions --

def validate(pxl: str) -> list[str]:
    """Validate a PXL string and return a list of warning/error messages.

    Each message is a human-readable string like::

        "line 3: ERROR: Invalid color \"#GGG\" for token x: ..."
        "line 5: WARNING: Undefined token y"

    An empty list means the input is valid.

    Args:
        pxl: PXL or JSONL format string to validate.

    Returns:
        List of validation messages. Empty if the input is valid.
    """
    ...

def validate_file(path: str) -> list[str]:
    """Validate a PXL file on disk and return a list of warning/error messages.

    Same output format as :func:`validate`, but reads from a file path.

    Args:
        path: Path to the ``.pxl`` file to validate.

    Returns:
        List of validation messages. Empty if the file is valid.

    Raises:
        OSError: If the file cannot be read.
    """
    ...

# -- Color utilities --

def parse_color(color_str: str) -> str:
    """Parse a CSS color string and return its hex representation.

    Accepts any format supported by the pixelsrc color parser: hex
    (``#f00``, ``#ff0000``), functional (``rgb()``, ``hsl()``, ``hwb()``,
    ``oklch()``), named (``red``, ``blue``, ``transparent``), and
    ``color-mix()``.

    Args:
        color_str: CSS color string to parse.

    Returns:
        Lowercase hex string. Colors with full opacity use ``#rrggbb``;
        colors with non-255 alpha use ``#rrggbbaa``.

    Raises:
        ValueError: If the color string is invalid.
    """
    ...

def generate_ramp(from_color: str, to_color: str, steps: int) -> list[str]:
    """Generate a color ramp by interpolating between two colors.

    The first element is *from_color* and the last is *to_color*, with
    intermediate colors evenly spaced in sRGB. Both endpoints are parsed
    with the full CSS color parser.

    Args:
        from_color: Starting color (any CSS color format).
        to_color: Ending color (any CSS color format).
        steps: Number of colors to generate (must be >= 1).

    Returns:
        List of hex color strings of length *steps*.

    Raises:
        ValueError: If either color is invalid or *steps* is zero.
    """
    ...

# -- PNG import functions --

def import_png(
    path: str,
    name: str | None = None,
    max_colors: int | None = None,
) -> ImportResult:
    """Import a PNG file and convert it to Pixelsrc format.

    Args:
        path: Path to the PNG file.
        name: Sprite name. Defaults to the filename stem.
        max_colors: Maximum palette size. Defaults to 16.

    Returns:
        An ``ImportResult`` with palette, grid, and region data.

    Raises:
        OSError: If the file cannot be read or is not a valid PNG.
    """
    ...

def import_png_analyzed(
    path: str,
    name: str | None = None,
    max_colors: int | None = None,
    confidence: float | None = None,
    hints: bool | None = None,
    shapes: bool | None = None,
    detect_upscale: bool | None = None,
    detect_outlines: bool | None = None,
    dither_handling: str | None = None,
) -> ImportResult:
    """Import a PNG file with full analysis options.

    Args:
        path: Path to the PNG file.
        name: Sprite name. Defaults to the filename stem.
        max_colors: Maximum palette size. Defaults to 16.
        confidence: Confidence threshold for analysis (0.0--1.0). Defaults to 0.5.
        hints: Generate token naming hints. Defaults to ``False``.
        shapes: Extract structured regions instead of raw points. Defaults to ``False``.
        detect_upscale: Detect if image is upscaled pixel art. Defaults to ``False``.
        detect_outlines: Detect outline/stroke regions. Defaults to ``False``.
        dither_handling: How to handle dither patterns: ``"keep"``, ``"merge"``,
            or ``"analyze"``. Defaults to ``"keep"``.

    Returns:
        An ``ImportResult`` with palette, grid, region data, and analysis results.

    Raises:
        OSError: If the file cannot be read or is not a valid PNG.
        ValueError: If *dither_handling* is not a recognized value.
    """
    ...

# -- Result types --

class RenderResult:
    """Result of rendering a sprite to RGBA pixels."""

    @property
    def width(self) -> int:
        """Width of the rendered image in pixels."""
        ...
    @property
    def height(self) -> int:
        """Height of the rendered image in pixels."""
        ...
    @property
    def pixels(self) -> bytes:
        """Raw RGBA pixel data (4 bytes per pixel)."""
        ...
    @property
    def warnings(self) -> list[str]:
        """Any warnings generated during rendering."""
        ...
    def to_numpy(self) -> np.ndarray:
        """Convert RGBA pixel data to a NumPy array with shape (height, width, 4).

        Requires the ``numpy`` package (``pip install pixelsrc[numpy]``).

        Returns:
            Array of dtype ``uint8`` with shape ``(height, width, 4)``.
        """
        ...
    def to_pil(self) -> Image.Image:
        """Convert RGBA pixel data to a PIL Image.

        Requires the ``Pillow`` package (``pip install pixelsrc[images]``).

        Returns:
            An RGBA image.
        """
        ...

class ImportResult:
    """Result of importing a PNG image into Pixelsrc format."""

    @property
    def name(self) -> str:
        """Name of the imported sprite."""
        ...
    @property
    def width(self) -> int:
        """Width of the imported image in pixels."""
        ...
    @property
    def height(self) -> int:
        """Height of the imported image in pixels."""
        ...
    @property
    def palette(self) -> dict[str, str]:
        """Color palette mapping token names to hex color strings."""
        ...
    @property
    def analysis(self) -> dict[str, Any] | None:
        """Analysis results, or ``None`` if analysis was not enabled.

        When present, contains keys: ``roles``, ``relationships``,
        ``symmetry``, ``naming_hints``, ``z_order``, ``dither_patterns``,
        ``upscale_info``, ``outlines``.
        """
        ...
    def to_pxl(self) -> str:
        """Convert to PXL grid format string (palette definition + grid rows)."""
        ...
    def to_jsonl(self) -> str:
        """Convert to JSONL format (palette line + sprite line)."""
        ...

# -- Stateful API --

class Registry:
    """A stateful registry that accumulates palettes and sprites from PXL content.

    Unlike the stateless ``render_to_png`` / ``render_to_rgba`` functions,
    ``Registry`` lets you load multiple PXL strings or files and then render
    any sprite by name.

    Example::

        from pixelsrc import Registry

        reg = Registry()
        reg.load_file("assets/palettes.pxl")
        reg.load_file("assets/sprites.pxl")

        print(reg.sprites())   # ['hero', 'enemy', ...]
        print(reg.palettes())  # ['warm', 'cool', ...]

        result = reg.render("hero")
        png = reg.render_to_png("hero")
        all_pngs = reg.render_all()
    """

    def __init__(self) -> None:
        """Create a new empty registry."""
        ...
    def load(self, pxl: str) -> None:
        """Load PXL/JSONL content from a string into the registry.

        Palettes and sprites are accumulated -- calling ``load`` multiple
        times adds to the existing registry contents.

        Args:
            pxl: PXL or JSONL format string to load.
        """
        ...
    def load_file(self, path: str) -> None:
        """Load PXL/JSONL content from a file path into the registry.

        Args:
            path: Path to a ``.pxl`` or ``.jsonl`` file.

        Raises:
            OSError: If the file cannot be read.
        """
        ...
    def sprites(self) -> list[str]:
        """Return a sorted list of sprite names in the registry."""
        ...
    def palettes(self) -> list[str]:
        """Return a sorted list of palette names in the registry."""
        ...
    def render(self, name: str) -> RenderResult:
        """Render a sprite by name and return RGBA pixel data.

        Args:
            name: Name of the sprite to render.

        Returns:
            A ``RenderResult`` with width, height, raw RGBA pixels, and warnings.

        Raises:
            ValueError: If the sprite is not found.
        """
        ...
    def render_to_png(self, name: str) -> bytes:
        """Render a sprite by name and return PNG image bytes.

        Args:
            name: Name of the sprite to render.

        Returns:
            PNG image data as bytes.

        Raises:
            ValueError: If the sprite is not found.
        """
        ...
    def render_all(self) -> dict[str, bytes]:
        """Render all sprites and return a dict mapping sprite name to PNG bytes."""
        ...
