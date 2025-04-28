cd "$(dirname "$0")"
if [ "$1" != "debug" ]; then
  cargo build --release
else
  cargo build
fi
cp target/release/BadgeChecker Checker
