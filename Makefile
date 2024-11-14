ci: fmt lint test

test: test-ink test-soroban test-substrate

fmt:
	@echo "Formatting Rust code..."
	@python3 scripts/run-fmt.py

lint:
	@echo "Linting cargo-scout-audit..."
	@python3 scripts/run-clippy.py

test-ink:
	@echo "Running ink tests..."
	@python3 scripts/find-test-cases.py -b=ink --format=list | xargs -I {} python3 scripts/run-tests.py --detector={}

test-soroban:
	@echo "Running soroban tests..."
	@python3 scripts/find-test-cases.py -b=soroban --format=list | xargs -I {} python3 scripts/run-tests.py --detector={}

test-substrate:
	@echo "Running substrate tests..."
	@python3 scripts/find-test-cases.py -b=substrate-pallets --format=list | xargs -I {} python3 scripts/run-tests.py --detector={}
