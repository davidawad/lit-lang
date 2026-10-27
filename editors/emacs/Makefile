# The litgraph Emacs package (editors/emacs) -- developer targets.
#
#   make test      ERT unit tests (process layer stubbed) plus integration
#                  tests against the real `litigate' binary when installed
#                  (LITGRAPH_IT_REQUIRE=1 fails instead of skipping; `just
#                  emacs-test' from the repository root runs it that way)
#   make compile   byte-compile with warnings as errors
#   make lint      checkdoc and package-lint
#   make all       compile, lint, test
#   make fixtures  regenerate test/fixtures/ from the `litigate' binary
#                  (LITIGATE=path; scripts/fixtures.sh)
#
# litigate-ts-mode's own suite (test/litigate-ts-mode-test.el) needs the
# tree-sitter grammar; `cargo nextest run -p litigate-editors' builds it
# and runs that suite.
#
# package-lint is looked up in PACKAGE_LINT_DIR, else in .deps/, else
# installed there from MELPA (needs network).

EMACS ?= emacs
BATCH  = $(EMACS) -Q --batch -L .
SRC    = litgraph.el litgraph-view.el litgraph-graph.el litgraph-org.el \
         litgraph-menu.el litgraph-timeline.el \
         ob-litgraph.el litigate-ts-mode-keywords.el litigate-ts-mode.el
# litigate-ts-mode (and its generated keyword lists) keeps its own `litigate-ts-mode-' prefix (it is
# usable on its own, and editors/litigate-editors tests it on its own),
# which package-lint would flag under this package's `litgraph-' prefix.
LINT_SRC = $(filter-out litigate-ts-mode.el litigate-ts-mode-keywords.el,$(SRC))
TESTS  = $(wildcard test/litgraph-*-test.el)
PACKAGE_LINT_DIR ?=
REPO   = ../..

.PHONY: all test compile lint checkdoc package-lint autoloads clean deps fixtures fixtures-sync

all: compile lint test

compile: clean-elc
	$(BATCH) --eval '(setq byte-compile-error-on-warn t)' \
	  -f batch-byte-compile $(SRC)

clean-elc:
	rm -f *.elc

test: clean-elc
	$(BATCH) -L test $(foreach t,$(TESTS),-l $(t)) \
	  -f ert-run-tests-batch-and-exit

lint: checkdoc package-lint

checkdoc:
	$(BATCH) -l scripts/lint.el -f litgraph-lint-checkdoc $(SRC)

package-lint: deps
	$(BATCH) -l scripts/lint.el --eval '(litgraph-lint-package-lint "$(PACKAGE_LINT_DIR)")' $(LINT_SRC)

deps:
	@true

LITIGATE ?= litigate

# Regenerate the engine fixtures from the real binary.
fixtures:
	LITIGATE="$(LITIGATE)" scripts/fixtures.sh

# Sync the timeline integration project from this repository's examples.
fixtures-sync:
	rm -rf test/fixtures/timeline-project/cases/examples test/fixtures/timeline-project/lit/examples
	mkdir -p test/fixtures/timeline-project/cases test/fixtures/timeline-project/lit
	cp -R "$(REPO)/cases/examples" test/fixtures/timeline-project/cases/
	cp -R "$(REPO)/lit/examples" test/fixtures/timeline-project/lit/

autoloads:
	$(BATCH) --eval '(loaddefs-generate default-directory "litgraph-autoloads.el")' || \
	$(BATCH) --eval '(make-directory-autoloads default-directory "litgraph-autoloads.el")'

clean:
	rm -f *.elc litgraph-autoloads.el
	rm -rf .deps
