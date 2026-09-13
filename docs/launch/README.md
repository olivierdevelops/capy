# Launch materials

Drafts of posts and announcements. Hold off on publishing until v0.1.0
has been tagged and the binaries are out.

| File | When |
|---|---|
| [blog-post.md](blog-post.md) | First. Goes on your personal site or dev.to. |
| [hn.md](hn.md) | Day of release. Tuesday or Wednesday morning PT. |
| [reddit.md](reddit.md) | Day of release. /r/rust, /r/ProgrammingLanguages. |
| [twitter.md](twitter.md) | Same day. Drop the link. |

Order of operations:

1. Tag `v0.1.0`. Confirm GoReleaser produced binaries for all platforms.
2. Verify `cargo install --git https://github.com/olivierdevelops/capy capy-cli` works.
3. Test the install script on a fresh Linux VM and a fresh macOS shell.
4. Publish the blog post.
5. Submit HN.
6. Cross-post to Reddit (separate posts; don't link-spam).
7. Tweet.
8. Stay engaged with comments for the first 4–6 hours.

Don't ask for upvotes anywhere. Don't simultaneously launch to 10 places —
the HN post benefits from being first.

After launch, watch:

- GitHub stars (vanity, but a sanity signal).
- Issues filed within 48 hours (the real signal — what surprised people).
- PRs (the very real signal — someone actually built something).
