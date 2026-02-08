"""Tests for optional NumPy/Pillow integration (PY-13)."""

import pytest

import pixelsrc


class TestToNumpy:
    """Tests for RenderResult.to_numpy()."""

    def test_returns_ndarray(self, minimal_sprite):
        np = pytest.importorskip("numpy")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        arr = result.to_numpy()
        assert isinstance(arr, np.ndarray)

    def test_shape_1x1(self, minimal_sprite):
        pytest.importorskip("numpy")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        arr = result.to_numpy()
        assert arr.shape == (1, 1, 4)

    def test_dtype_uint8(self, minimal_sprite):
        np = pytest.importorskip("numpy")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        arr = result.to_numpy()
        assert arr.dtype == np.uint8

    def test_red_pixel_values(self, minimal_sprite):
        pytest.importorskip("numpy")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        arr = result.to_numpy()
        assert arr[0, 0, 0] == 255  # R
        assert arr[0, 0, 1] == 0    # G
        assert arr[0, 0, 2] == 0    # B
        assert arr[0, 0, 3] == 255  # A

    def test_shape_4x4(self, heart_with_palette):
        pytest.importorskip("numpy")
        result = pixelsrc.render_to_rgba(heart_with_palette)
        arr = result.to_numpy()
        assert arr.shape == (4, 4, 4)

    def test_array_is_writable(self, minimal_sprite):
        pytest.importorskip("numpy")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        arr = result.to_numpy()
        arr[0, 0, 0] = 42  # should not raise

    def test_missing_numpy_raises(self, minimal_sprite, monkeypatch):
        import builtins

        real_import = builtins.__import__

        def mock_import(name, *args, **kwargs):
            if name == "numpy":
                raise ModuleNotFoundError("No module named 'numpy'")
            return real_import(name, *args, **kwargs)

        result = pixelsrc.render_to_rgba(minimal_sprite)
        monkeypatch.setattr(builtins, "__import__", mock_import)
        with pytest.raises(ModuleNotFoundError):
            result.to_numpy()


class TestToPil:
    """Tests for RenderResult.to_pil()."""

    def test_returns_pil_image(self, minimal_sprite):
        PIL = pytest.importorskip("PIL")
        from PIL import Image

        result = pixelsrc.render_to_rgba(minimal_sprite)
        img = result.to_pil()
        assert isinstance(img, Image.Image)

    def test_mode_rgba(self, minimal_sprite):
        pytest.importorskip("PIL")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        img = result.to_pil()
        assert img.mode == "RGBA"

    def test_size_1x1(self, minimal_sprite):
        pytest.importorskip("PIL")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        img = result.to_pil()
        assert img.size == (1, 1)

    def test_red_pixel_value(self, minimal_sprite):
        pytest.importorskip("PIL")
        result = pixelsrc.render_to_rgba(minimal_sprite)
        img = result.to_pil()
        assert img.getpixel((0, 0)) == (255, 0, 0, 255)

    def test_size_4x4(self, heart_with_palette):
        pytest.importorskip("PIL")
        result = pixelsrc.render_to_rgba(heart_with_palette)
        img = result.to_pil()
        assert img.size == (4, 4)

    def test_missing_pillow_raises(self, minimal_sprite, monkeypatch):
        import builtins

        real_import = builtins.__import__

        def mock_import(name, *args, **kwargs):
            if name == "PIL" or name == "PIL.Image":
                raise ModuleNotFoundError("No module named 'PIL'")
            return real_import(name, *args, **kwargs)

        result = pixelsrc.render_to_rgba(minimal_sprite)
        monkeypatch.setattr(builtins, "__import__", mock_import)
        with pytest.raises(ModuleNotFoundError):
            result.to_pil()


class TestNumpyPilRoundtrip:
    """Test NumPy <-> PIL roundtrip conversion."""

    def test_numpy_to_pil_roundtrip(self, heart_with_palette):
        np = pytest.importorskip("numpy")
        PIL = pytest.importorskip("PIL")
        from PIL import Image

        result = pixelsrc.render_to_rgba(heart_with_palette)
        arr = result.to_numpy()
        img = Image.fromarray(arr, "RGBA")
        assert img.size == (4, 4)

        arr2 = np.array(img)
        np.testing.assert_array_equal(arr, arr2)
