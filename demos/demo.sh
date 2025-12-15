#!/bin/bash
# Caro Demo Manager
# Manage asciinema recordings for different use cases

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ASCIINEMA_DIR="$SCRIPT_DIR/asciinema"
PLAYGROUND_DIR="$SCRIPT_DIR/playground"
SYSADMIN_DIR="$SCRIPT_DIR/sysadmin-playground"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Check if binary exists
check_binary() {
    if [ ! -f "$PROJECT_ROOT/target/release/cmdai" ]; then
        print_error "Binary not found. Building..."
        cd "$PROJECT_ROOT"
        cargo build --release --features embedded-mlx
    fi
    print_success "Binary ready"
}

# Check if asciinema is installed
check_asciinema() {
    if ! command -v asciinema &> /dev/null; then
        print_error "asciinema not installed"
        echo "Install with: brew install asciinema"
        exit 1
    fi
}

# Check if agg is installed (for GIF generation)
check_agg() {
    if ! command -v agg &> /dev/null; then
        print_info "agg not installed (optional for GIF generation)"
        echo "Install with: cargo install agg"
        return 1
    fi
    return 0
}

get_demo_name() {
    case "$1" in
        vancouver) echo "Vancouver.Dev Event" ;;
        website) echo "Website Hero" ;;
        sysadmin) echo "SysAdmin/DevOps" ;;
        *) echo "Unknown" ;;
    esac
}

get_demo_purpose() {
    case "$1" in
        vancouver) echo "For tomorrow's community presentation" ;;
        website) echo "For caro.sh homepage (repo: wildcard/caro)" ;;
        sysadmin) echo "For technical operations audience" ;;
        *) echo "" ;;
    esac
}

get_demo_workdir() {
    case "$1" in
        vancouver) echo "$PLAYGROUND_DIR" ;;
        website) echo "$PLAYGROUND_DIR" ;;
        sysadmin) echo "$SYSADMIN_DIR" ;;
        *) echo "" ;;
    esac
}

get_demo_script() {
    case "$1" in
        vancouver) echo "vancouver-dev-demo.sh" ;;
        website) echo "website-hero-demo.sh" ;;
        sysadmin) echo "sysadmin-demo.sh" ;;
        *) echo "" ;;
    esac
}

show_demos() {
    print_header "Available Demos"
    echo ""
    
    for id in vancouver website sysadmin; do
        echo -e "${GREEN}$id${NC}"
        echo -e "  Name:    ${BLUE}$(get_demo_name $id)${NC}"
        echo -e "  Purpose: $(get_demo_purpose $id)"
        echo -e "  Script:  $(get_demo_script $id)"
        echo ""
    done
}

# Record demo
record_demo() {
    local demo_id="$1"
    check_binary
    check_asciinema
    
    local name=$(get_demo_name "$demo_id")
    local workdir=$(get_demo_workdir "$demo_id")
    local script=$(get_demo_script "$demo_id")
    
    if [ -z "$script" ]; then
        print_error "Unknown demo: $demo_id"
        show_demos
        exit 1
    fi
    
    print_header "Recording: $name"
    print_info "Purpose: $(get_demo_purpose $demo_id)"
    print_info "Working directory: $workdir"
    echo ""
    
    cd "$workdir"
    local output_file="$ASCIINEMA_DIR/${demo_id}-demo.cast"
    
    print_info "Recording will start in 3 seconds..."
    sleep 3
    
    asciinema rec "$output_file" -c "$ASCIINEMA_DIR/$script" --overwrite
    
    print_success "Recording saved to: $output_file"
}

# Play demo
play_demo() {
    local demo_id="$1"
    check_asciinema
    
    local cast_file="$ASCIINEMA_DIR/${demo_id}-demo.cast"
    
    if [ ! -f "$cast_file" ]; then
        print_error "Recording not found: $cast_file"
        echo "Record it first with: $0 record $demo_id"
        exit 1
    fi
    
    print_header "Playing: $(get_demo_name $demo_id)"
    asciinema play "$cast_file"
}

# Upload demo to asciinema.org
upload_demo() {
    local demo_id="$1"
    check_asciinema
    
    local cast_file="$ASCIINEMA_DIR/${demo_id}-demo.cast"
    
    if [ ! -f "$cast_file" ]; then
        print_error "Recording not found: $cast_file"
        exit 1
    fi
    
    print_header "Uploading: $(get_demo_name $demo_id)"
    asciinema upload "$cast_file"
}

# Generate GIF from recording
generate_gif() {
    local demo_id="$1"
    
    if ! check_agg; then
        print_error "Cannot generate GIF without agg"
        exit 1
    fi
    
    local cast_file="$ASCIINEMA_DIR/${demo_id}-demo.cast"
    local gif_file="$ASCIINEMA_DIR/${demo_id}-demo.gif"
    
    if [ ! -f "$cast_file" ]; then
        print_error "Recording not found: $cast_file"
        exit 1
    fi
    
    print_header "Generating GIF: $(get_demo_name $demo_id)"
    
    # Generate with good quality settings
    agg --speed 1.5 --font-size 14 "$cast_file" "$gif_file"
    
    print_success "GIF saved to: $gif_file"
    
    # Show file size
    local size=$(du -h "$gif_file" | cut -f1)
    print_info "File size: $size"
}

# Record all demos
record_all() {
    print_header "Recording All Demos"
    for id in vancouver website sysadmin; do
        echo ""
        record_demo "$id"
    done
}

# Generate all GIFs
generate_all_gifs() {
    print_header "Generating All GIFs"
    for id in vancouver website sysadmin; do
        echo ""
        generate_gif "$id"
    done
}

# Show usage
usage() {
    cat << EOF
Caro Demo Manager

USAGE:
    $0 <command> [demo-id]

COMMANDS:
    list              List all available demos
    record <id>       Record a demo
    play <id>         Play back a recording
    upload <id>       Upload demo to asciinema.org
    gif <id>          Generate GIF from recording
    record-all        Record all demos
    gif-all           Generate GIFs for all demos

DEMO IDs:
    vancouver         Vancouver.Dev event presentation (tomorrow)
    website           Website hero section for caro.sh
    sysadmin          SysAdmin/DevOps/SRE audience

EXAMPLES:
    # Record Vancouver demo
    $0 record vancouver
    
    # Play it back
    $0 play vancouver
    
    # Generate GIF
    $0 gif vancouver
    
    # Upload to asciinema.org
    $0 upload vancouver
    
    # Record all demos
    $0 record-all

REPOSITORY:
    Website demo is for: github.com/wildcard/caro
    Domain: caro.sh

REQUIREMENTS:
    - asciinema: brew install asciinema
    - agg (for GIFs): cargo install agg

EOF
}

# Main command dispatcher
main() {
    if [ $# -eq 0 ]; then
        usage
        exit 0
    fi
    
    local command="$1"
    shift
    
    case "$command" in
        list)
            show_demos
            ;;
        record)
            if [ $# -eq 0 ]; then
                print_error "Demo ID required"
                echo "Usage: $0 record <demo-id>"
                exit 1
            fi
            record_demo "$1"
            ;;
        play)
            if [ $# -eq 0 ]; then
                print_error "Demo ID required"
                exit 1
            fi
            play_demo "$1"
            ;;
        upload)
            if [ $# -eq 0 ]; then
                print_error "Demo ID required"
                exit 1
            fi
            upload_demo "$1"
            ;;
        gif)
            if [ $# -eq 0 ]; then
                print_error "Demo ID required"
                exit 1
            fi
            generate_gif "$1"
            ;;
        record-all)
            record_all
            ;;
        gif-all)
            generate_all_gifs
            ;;
        *)
            print_error "Unknown command: $command"
            usage
            exit 1
            ;;
    esac
}

main "$@"
