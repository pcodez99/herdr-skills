CARGO ?= cargo
RELEASE_BIN := target/release/herdr-plans
ARCHIVE_BIN := skills/herdr-grill/bin/herdr-plans
FILES := .claude-plugin/plugin.json hooks/hooks.json hooks/activate.sh \
	skills/herdr-team/SKILL.md skills/herdr-team/TEAMMATE.md \
	skills/herdr-team/LIFECYCLE.md skills/herdr-team/TROUBLESHOOTING.md skills/herdr-team/WIKI.md \
	skills/herdr-grill/SKILL.md skills/herdr-grill/plans assets/icon.png LICENSE README.md PRIVACY.md
ZIP := dist/herdr-claude.zip

.PHONY: build release check package check-package

build:
	$(CARGO) build --locked --target-dir target

release:
	$(CARGO) build --locked --release --target-dir target

check: build
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --locked --all-targets --target-dir target -- -D warnings
	$(CARGO) test --locked --target-dir target
	sh -n hooks/activate.sh skills/herdr-grill/plans
	test -z "$$(env -u HC_TERMINAL_ID sh hooks/activate.sh)"
	HC_TERMINAL_ID=t-test sh hooks/activate.sh | grep -q 'address t-test\.'
	claude plugin validate . --strict --profile directory

package:
	@test -x "$(RELEASE_BIN)" || { echo 'Missing executable $(RELEASE_BIN). Run make release on macOS arm64.' >&2; exit 1; }
	@file -b "$(RELEASE_BIN)" | grep -q '^Mach-O 64-bit executable arm64' || { echo 'Expected a native macOS arm64 executable at $(RELEASE_BIN).' >&2; exit 1; }
	@mkdir -p dist && rm -f $(ZIP)
	@stage=$$(mktemp -d) && trap 'rm -rf "$$stage"' EXIT && \
		mkdir -p "$$stage/$$(dirname $(ARCHIVE_BIN))" && \
		install -m 755 "$(RELEASE_BIN)" "$$stage/$(ARCHIVE_BIN)" && \
		zip -q $(ZIP) $(FILES) && \
		(cd "$$stage" && zip -q "$(CURDIR)/$(ZIP)" "$(ARCHIVE_BIN)")

# Runs the CLI tests through the launcher of the extracted ZIP, with only Git on PATH.
check-package: package
	@dir=$$(mktemp -d) && trap 'rm -rf "$$dir"' EXIT && \
		unzip -q $(ZIP) -d "$$dir/pkg" && \
		for f in $(FILES) $(ARCHIVE_BIN); do test -e "$$dir/pkg/$$f" || { echo "Missing $$f" >&2; exit 1; }; done && \
		test -x "$$dir/pkg/$(ARCHIVE_BIN)" && test -x "$$dir/pkg/skills/herdr-grill/plans" && \
		cmp -s "$$dir/pkg/$(ARCHIVE_BIN)" "$(RELEASE_BIN)" && \
		mkdir "$$dir/bin" && ln -s "$$(command -v git)" "$$dir/bin/git" && \
		PLANS_BIN="$$dir/pkg/skills/herdr-grill/plans" PLANS_PATH="$$dir/bin" \
		$(CARGO) test --locked --target-dir target --test cli
