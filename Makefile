.PHONY: package check

package:
	mkdir -p dist
	rm -f dist/herdr-codex.zip dist/herdr-claude.zip
	zip -q -r dist/herdr-codex.zip plugin.json skills/herdr-team assets/icon.png LICENSE README.md PRIVACY.md
	zip -q -r dist/herdr-claude.zip .claude-plugin/plugin.json skills/herdr-team hooks assets/icon.png LICENSE README.md PRIVACY.md

check:
	python3 -m unittest discover -s tests
	claude plugin validate . --strict --profile directory
