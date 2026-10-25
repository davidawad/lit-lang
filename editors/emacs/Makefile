# litgraph-emacs -- developer targets.
#
#   make test      ERT unit tests (process layer stubbed) plus integration
#                  tests against the real `litgraph' binary when installed
#   make compile   byte-compile with warnings as errors
#   make lint      checkdoc and package-lint
#   make all       compile, lint, test
#
# package-lint is looked up in PACKAGE_LINT_DIR, else in .deps/, else
# installed there from MELPA (needs network).

EMACS ?= emacs
BATCH  = $(EMACS) -Q --batch -L .
SRC    = litgraph.el litgraph-view.el litgraph-graph.el litgraph-org.el \
         litgraph-scenario.el litgraph-menu.el litgraph-timeline.el \
         ob-litgraph.el
TESTS  = $(wildcard test/*-test.el)
PACKAGE_LINT_DIR ?=
LITGRAPH_DIR ?=
LITGRAPH_SHA ?= 527253abd5bf74af700b9bd32fc4f4aa26b072c5

.PHONY: all test compile lint checkdoc package-lint autoloads clean deps fixtures-sync

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
	$(BATCH) -l scripts/lint.el --eval '(litgraph-lint-package-lint "$(PACKAGE_LINT_DIR)")' $(SRC)

deps:
	@true

# Sync the real timeline integration project from the pinned litgraph checkout.
fixtures-sync:
	@test -n "$(LITGRAPH_DIR)" || { echo "set LITGRAPH_DIR to a litgraph checkout" >&2; exit 2; }
	@test "$$(git -C "$(LITGRAPH_DIR)" rev-parse HEAD)" = "$(LITGRAPH_SHA)" || { echo "litgraph checkout must be at $(LITGRAPH_SHA)" >&2; exit 2; }
	rm -rf test/fixtures/timeline-project/cases/examples test/fixtures/timeline-project/lit/examples
	mkdir -p test/fixtures/timeline-project/cases test/fixtures/timeline-project/lit
	cp -R "$(LITGRAPH_DIR)/cases/examples" test/fixtures/timeline-project/cases/
	cp -R "$(LITGRAPH_DIR)/lit/examples" test/fixtures/timeline-project/lit/

autoloads:
	$(BATCH) --eval '(loaddefs-generate default-directory "litgraph-autoloads.el")' || \
	$(BATCH) --eval '(make-directory-autoloads default-directory "litgraph-autoloads.el")'

clean:
	rm -f *.elc litgraph-autoloads.el
	rm -rf .deps
