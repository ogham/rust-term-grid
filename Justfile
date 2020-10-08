all: build test
all-release: build-release test-release

MIN_RUST := "1.31.0"


# compiles the code
build:
    cargo +{{MIN_RUST}} build
    cargo +stable       build

# compiles the code in release mode
build-release:
    cargo +{{MIN_RUST}} build --release --verbose
    cargo +stable       build --release --verbose


# runs unit tests
test:
    cargo +{{MIN_RUST}} test --all -- --quiet
    cargo +stable       test --all -- --quiet

# runs unit tests in release mode
test-release:
    cargo +{{MIN_RUST}} test --all --release --verbose
    cargo +stable       test --all --release --verbose
