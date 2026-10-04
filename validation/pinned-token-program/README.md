# Pinned legacy Token executable

This separate workspace builds the published `spl-token 8.0.0` cdylib directly.
The initial combined graph enabled Token's `no-entrypoint` feature through the
stake-pool dependency graph, even for a package-selected build. That rejected
artifact and all first-attempt evidence remain preserved. Independent resolution
prevents the pool graph from suppressing this executable's entrypoint. Root checks
the locked source closure, active feature graph and actual ELF entrypoint before
runtime use. No processor wrapper, source patch or production dependency change
is introduced. The pool workspace retains its own independently reviewed lock.
