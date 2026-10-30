#!/bin/sh
# Puts the org agenda of today into the panel `today` of the scene `agenda`
# of chora-mc. Emacs builds the agenda, so repeats, warnings and overdue
# entries come out as in Emacs; the files are only read.
#
# Usage: agenda.sh [ORG_FILE...]   (default: ~/Notes/agenda/gtd.org)
# Run it from a timer, or with --print to see the JSON without chora.
set -eu
print=
if [ "${1-}" = --print ]; then
    print=1
    shift
fi
[ $# -gt 0 ] || set -- "$HOME/Notes/agenda/gtd.org"
params=$(emacs -Q --batch --eval '
(progn
  (require (quote org-agenda))
  (require (quote json))
  (setq org-agenda-files command-line-args-left
        command-line-args-left nil
        org-agenda-prefix-format "%?-12t%s"
        org-agenda-todo-keyword-format ""
        org-agenda-remove-tags t
        org-agenda-use-time-grid nil)
  (org-agenda-list nil nil 1)
  ;; The first two lines are the header and the date.
  (let ((lines (cddr (split-string (buffer-substring-no-properties (point-min) (point-max))
                                  "\n" t "[ \t]+"))))
    (princ (json-encode
            (list (cons "scene" "agenda") (cons "panel" "today")
                  (cons "title" (format-time-string "%A, %e %B"))
                  (cons "lines" (vconcat lines)))))))' "$@" 2>/dev/null)
if [ -n "$print" ]; then
    printf '%s\n' "$params"
else
    choractl call mc panel.set "$params" >/dev/null
fi
