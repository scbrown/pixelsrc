# Exit Codes

Pixelsrc CLI commands return standard exit codes to indicate success or failure. These codes are useful for scripting and CI/CD integration.

## Exit Code Summary

| Code | Name | Description |
|------|------|-------------|
| 0 | Success | Command completed successfully |
| 1 | Error | Runtime error during execution |
| 2 | Invalid Arguments | Invalid command-line arguments |

## Exit Code Details

### 0 - Success

The command completed without errors.

```bash
pxl render sprite.pxl -o sprite.png
echo $?  # 0
```

### 1 - Error

A runtime error occurred during execution. Common causes:

- File not found or unreadable
- Invalid JSON/JSONL syntax
- Missing required fields in definitions
- Color parsing errors
- Render failures
- I/O errors writing output files

```bash
pxl render nonexistent.pxl -o out.png
echo $?  # 1
# Error: failed to read input file: No such file or directory
```

### 2 - Invalid Arguments

Command-line arguments are invalid or missing required values. Common causes:

- Missing required arguments
- Invalid option values
- Mutually exclusive options used together
- Unknown subcommand or option

```bash
pxl render
echo $?  # 2
# Error: the following required arguments were not provided: <INPUT>
```

## Using Exit Codes in Scripts

### Bash

```bash
#!/bin/bash
if pxl validate sprites/*.pxl; then
    echo "All sprites valid"
    pxl build
else
    echo "Validation failed"
    exit 1
fi
```

### Make

```makefile
sprites: $(wildcard src/*.pxl)
	pxl build || exit 1

validate:
	pxl validate src/*.pxl
```

### CI/CD (GitHub Actions)

```yaml
- name: Validate sprites
  run: pxl validate src/**/*.pxl

- name: Build assets
  run: pxl build
  # Fails the job if exit code != 0
```

## Command-Specific Behavior

### validate

Returns exit code 1 if any validation errors are found:

```bash
pxl validate sprite.pxl
# Exit 0: No errors
# Exit 1: Validation errors found
```

With `--strict`, warnings also cause exit code 1.

### build

Returns exit code 1 if any file fails to build:

```bash
pxl build
# Exit 0: All files built successfully
# Exit 1: One or more files failed
```

### diff

Returns exit code 0 even when differences are found (differences are not errors):

```bash
pxl diff a.pxl b.pxl
# Exit 0: Comparison completed (may have differences)
# Exit 1: Error reading files
```

### render

Returns exit code 1 for any render failure:

```bash
pxl render sprite.pxl -o out.png
# Exit 0: Rendered successfully
# Exit 1: Render failed
```

## Error Output

Error messages are written to stderr, allowing you to separate them from normal output:

```bash
# Capture errors separately
pxl validate sprites/*.pxl 2>errors.txt

# Suppress errors
pxl validate sprites/*.pxl 2>/dev/null
```

## Related

- [validate command](../cli/validate.md) - Validation options
- [build command](../cli/build.md) - Build system
- [Configuration](config.md) - Build configuration
