# Questions for the user

Each: number, date, background, default taken.

1. 2026-11-09. D75 (f5f346f) keeps "a depth buffer in the core" rejected
   and narrows the rejected row instead of marking it superseded, as the
   charter said. Reason: sorting and splitting planes handles translucent
   pane edges, which a depth buffer does not, and Smithay's framebuffers
   have no depth attachment; dense 3D geometry belongs in clients.
   Default taken: keep it rejected. Answer "allow a depth buffer" to
   reopen it.
2. 2026-11-09. 83f9b6c marks the chora-mc methods `agent.set` and
   `notify.push` privileged in the core. Voice already asked before them;
   new is that other clients (for example an agent on a client socket)
   now need the grant `methods.privileged` to call them. Default taken:
   keep it. Answer "unprivileged" to revert.
