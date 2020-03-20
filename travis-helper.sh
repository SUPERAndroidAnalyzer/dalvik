#!/bin/bash

action="$1"

# Run unit and integration tests.
if [ "$action" = "test" ]; then
  cargo test --verbose

# Check formatting.
elif [ "$action" = "fmt_check" ]; then
  if [[ "$TRAVIS_RUST_VERSION" == "stable" && "$TRAVIS_OS_NAME" == "linux" ]]; then
    rustup component add rustfmt &&
    cargo fmt --verbose -- --check
  fi

# Run Clippy.
elif [ "$action" = "clippy_check" ]; then
  if [[ "$TRAVIS_RUST_VERSION" == "stable" && "$TRAVIS_OS_NAME" == "linux" ]]; then
    rustup component add clippy &&
    cargo clippy --verbose
  fi

# Upload code coverage report for stable builds in Linux.
elif [ "$action" = "upload_code_coverage" ]; then
  if [[ "$TRAVIS_BUILD_STAGE_NAME" == "Test" &&
        "$TRAVIS_RUST_VERSION" == "stable" &&
        "$TRAVIS_OS_NAME" == "linux" ]]; then
    cargo install cargo-tarpaulin &&
    cargo tarpaulin --out Xml -v &&
    bash <(curl -s https://codecov.io/bash) &&
    echo "Uploaded code coverage"
  fi

# Upload development documentation for the develop branch.
elif [ "$action" = "documentation" ]; then
  cargo doc -v --document-private-items &&
  echo "<meta http-equiv=refresh content=0;url=dalvik/index.html>" > target/doc/index.html

fi
exit $?