#!/bin/sh
# Puts the org agenda of today into the panel `today` of the scene
# `agenda` of chora-mc: the entries with a time of today, and the tasks
# scheduled or due today. Nothing overdue, nothing upcoming, nothing done.
# Each line is "TIME<tab>TEXT"; an entry without a time has an empty time.
# The next entry with a time goes into the content `next` of the regions,
# for the footer; nothing when none is left today.
# Emacs builds the agenda, so repeats come out as in Emacs; the files are
# only read.
#
# Usage: agenda.sh [--print] [ORG_FILE...]   (default: ~/Notes/agenda/gtd.org)
# Run it from a timer, or with --print to see the JSON without chora.
# ORG_TODO sets the TODO keywords, as in #+TODO, when the files have none.
set -eu
print=
if [ "${1-}" = --print ]; then
    print=1
    shift
fi
[ $# -gt 0 ] || set -- "$HOME/Notes/agenda/gtd.org"
command -v emacs >/dev/null || {
    echo "agenda.sh: no emacs on PATH" >&2
    exit 1
}
# A missing file makes org-agenda ask "[R]emove from list or [A]bort?",
# and a batch Emacs then waits for ever, even on a closed stdin: check
# first. Org reads a relative file name against org-directory, not the
# current directory, so the Lisp expands each name.
for f; do
    [ -r "$f" ] || {
        echo "agenda.sh: cannot read $f" >&2
        exit 1
    }
done
# Emacs talks on stderr even when it works: only its status counts. The
# timeout ends an Emacs that waits on a prompt all the same.
params=$(ORG_TODO="${ORG_TODO-PROJ TODO NEXT WAIT | DONE KILL}" timeout 60 emacs -Q --batch --eval '
(progn
  (require (quote org-agenda))
  (require (quote json))
  (setq org-agenda-files (mapcar (function expand-file-name) command-line-args-left)
        org-agenda-skip-unavailable-files t
        command-line-args-left nil
        org-todo-keywords (list (cons (quote sequence) (split-string (getenv "ORG_TODO"))))
        org-agenda-use-time-grid nil
        org-agenda-skip-scheduled-if-done t
        org-agenda-skip-deadline-if-done t
        org-agenda-skip-timestamp-if-done t)
  (org-agenda-list nil nil 1)
  (let ((today (org-today)) items)
    (goto-char (point-min))
    (while (not (eobp))
      (let* ((p (point))
             (type (get-text-property p (quote type)))
             (txt (get-text-property p (quote txt)))
             (state (get-text-property p (quote todo-state)))
             (due (get-text-property p (quote ts-date))))
        (when (and txt
                   (member type (quote ("timestamp" "block" "scheduled" "deadline")))
                   (or (member type (quote ("timestamp" "block"))) (eql due today))
                   (not (member state org-done-keywords-for-agenda)))
          (let ((time (string-trim (replace-regexp-in-string
                                    "\\.+$" "" (or (get-text-property p (quote time)) ""))))
                (text (substring-no-properties txt)))
            (when (and state (string-prefix-p (concat state " ") text))
              (setq text (substring text (1+ (length state)))))
            (setq text (string-trim (replace-regexp-in-string
                                     "[ \t]+:[[:alnum:]_@#%:]+:[ \t]*$" "" text)))
            (push (cons time text) items))))
      (forward-line 1))
    (setq items (nreverse items))
    ;; chora-mc shows the part before the tab as a column of its own.
    (princ (json-encode
            (list (cons "scene" "agenda") (cons "panel" "today")
                  (cons "lines" (vconcat
                                 (mapcar (lambda (i) (concat (car i) "\t" (cdr i)))
                                         items))))))
    ;; The next entry whose start is not over, times as HH:MM.
    (let* ((pad (lambda (time) (if (string-match "^[0-9]:" time) (concat "0" time) time)))
           (now (format-time-string "%H:%M"))
           (next (seq-find (lambda (i) (and (> (length (car i)) 0)
                                            (not (string< (funcall pad (car i)) now))))
                           items)))
      (princ "\n")
      (princ (json-encode
              (list (cons "content" "next")
                    (cons "lines" (vconcat (and next
                                                (list (concat (car next) "  " (cdr next))))))))))))' "$@" </dev/null 2>/dev/null) || {
    echo "agenda.sh: emacs failed with status $?" >&2
    exit 1
}
if [ -n "$print" ]; then
    printf '%s\n' "$params"
else
    choractl call mc panel.set "$(printf '%s\n' "$params" | sed -n 1p)" >/dev/null
    choractl call mc content.set "$(printf '%s\n' "$params" | sed -n 2p)" >/dev/null
fi
