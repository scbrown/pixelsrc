#!/usr/bin/awk -f
# Parse @demo annotations from Rust test files
# Output: TAB-separated records: DEMO<TAB>demo_path<TAB>title<TAB>description<TAB>cli<TAB>jsonl_path

BEGIN {
    in_demo = 0
    found_fn = 0
    demo = ""
    title = ""
    desc = ""
    cli = ""
    jsonl = ""
}

# Start of @demo annotation
/^[[:space:]]*\/\/\/[[:space:]]*@demo[[:space:]]/ {
    # Emit previous demo if pending
    if (length(demo) > 0 && length(jsonl) > 0) {
        printf "DEMO\t%s\t%s\t%s\t%s\t%s\n", demo, title, desc, cli, jsonl
    }
    in_demo = 1
    found_fn = 0
    demo = $0
    sub(/^[[:space:]]*\/\/\/[[:space:]]*@demo[[:space:]]*/, "", demo)
    title = ""
    desc = ""
    cli = ""
    jsonl = ""
    next
}

# Parse @title annotation
in_demo == 1 && /^[[:space:]]*\/\/\/[[:space:]]*@title[[:space:]]/ {
    title = $0
    sub(/^[[:space:]]*\/\/\/[[:space:]]*@title[[:space:]]*/, "", title)
    next
}

# Parse @description annotation
in_demo == 1 && /^[[:space:]]*\/\/\/[[:space:]]*@description[[:space:]]/ {
    desc = $0
    sub(/^[[:space:]]*\/\/\/[[:space:]]*@description[[:space:]]*/, "", desc)
    next
}

# Parse @cli annotation
in_demo == 1 && /^[[:space:]]*\/\/\/[[:space:]]*@cli[[:space:]]/ {
    cli = $0
    sub(/^[[:space:]]*\/\/\/[[:space:]]*@cli[[:space:]]*/, "", cli)
    next
}

# Regular doc comment (not @-prefixed) - use as description if no @description found
in_demo == 1 && found_fn == 0 && /^[[:space:]]*\/\/\/[[:space:]]/ {
    if (!/^[[:space:]]*\/\/\/[[:space:]]*@/) {
        # Non-annotation doc comment
        line = $0
        sub(/^[[:space:]]*\/\/\/[[:space:]]*/, "", line)
        if (length(title) == 0 && length(desc) == 0) {
            # First non-annotation comment becomes title
            title = line
        } else if (length(desc) == 0) {
            # Second becomes description start
            desc = line
        } else {
            # Append to description
            desc = desc " " line
        }
        next
    }
}

# Skip #[test] attribute
in_demo == 1 && /^[[:space:]]*#\[test\]/ {
    next
}

# Track function definition
in_demo == 1 && /^[[:space:]]*fn[[:space:]]+test_/ {
    found_fn = 1
    next
}

# Find include_str! with JSONL path (comes after fn)
in_demo == 1 && found_fn == 1 && /include_str!\(/ {
    if (/\.jsonl/) {
        jsonl = $0
        sub(/.*include_str!\([[:space:]]*"/, "", jsonl)
        sub(/".*/, "", jsonl)
        sub(/^\.\.\/\.\.\//, "", jsonl)
        # We got everything - emit and reset
        # Use <EMPTY> placeholder for empty fields (bash read can't handle consecutive delimiters)
        if (length(demo) > 0) {
            out_title = (length(title) > 0) ? title : "<EMPTY>"
            out_desc = (length(desc) > 0) ? desc : "<EMPTY>"
            out_cli = (length(cli) > 0) ? cli : "<EMPTY>"
            printf "DEMO\t%s\t%s\t%s\t%s\t%s\n", demo, out_title, out_desc, out_cli, jsonl
        }
        in_demo = 0
        found_fn = 0
        demo = ""
    }
}

# Closing brace after function - reset if no jsonl found
in_demo == 1 && found_fn == 1 && /^[[:space:]]*\}/ {
    # Did not find include_str!, skip this demo
    in_demo = 0
    found_fn = 0
    demo = ""
}

END {
    # Emit any pending demo
    if (length(demo) > 0 && length(jsonl) > 0) {
        printf "DEMO\t%s\t%s\t%s\t%s\t%s\n", demo, title, desc, cli, jsonl
    }
}
