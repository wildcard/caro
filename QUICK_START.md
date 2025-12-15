# Quick Start - Running Demos

## 1. Build the Binary

```bash
cd /Users/kobi/personal/cmdai
cargo build --release --features embedded-mlx
```

## 2. Run Demos Directly

### Vancouver Dev Demo (full presentation)
```bash
cd demos/playground
bash ../vancouver-dev-demo.sh
```

### Website Hero Demo (quick 1-min)
```bash
cd demos/playground
bash ../website-hero-demo.sh
```

### SysAdmin Demo
```bash
cd demos/sysadmin-playground
bash ../sysadmin-demo.sh
```

## 3. Record with Asciinema

```bash
cd demos/playground
asciinema rec vancouver-demo.cast -c "../vancouver-dev-demo.sh"

# Upload to asciinema.org
asciinema upload vancouver-demo.cast
```

## 4. Test cmdai Directly

```bash
# From project root
./target/release/cmdai "list all files"

# With execution
./target/release/cmdai -x "list all files"

# Verbose mode (shows logs)
./target/release/cmdai -v "list all files"
```

## Notes

- First run will be slow (model loading ~2-3s)
- Subsequent runs are fast (<1s)
- Demo scripts automatically find the binary
- All output is clean and professional
