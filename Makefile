APP := tuipe

build:
	@printf "\e[36m==> \e[0mCompiling...\n"
	@cargo build --release

run:
	@printf "\e[36m==> \e[0mStarting...\n"
	@cargo run

clean:
	@printf "\e[36m==> \e[0mCleaning up...\n"
	@cargo clean
