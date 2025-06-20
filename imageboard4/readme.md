# Chessboard: A Focused, Secure, Self-Hosted Educational Forum

## What is Chessboard?

**Chessboard** is a simple, ultra-reliable message board system designed for educators, clubs, teams, or anyone who wants a secure, private, and transparent place for organized discussions. Built with Rust and SQLite, it is robust by design and prioritizes administrator control and content clarity over flashy features.

---

## How Does Chessboard Work?

* **Boards are just directories.**

  * To add a new discussion board: create a new folder under `/chess/` (e.g. `/chess/logic101` or `/chess/calculus`).
  * The first visit to a new board auto-creates a database and uploads folder in that directory.
  * To delete a board: simply delete its folder—posts, files, and data are instantly and safely removed.
* **All content is stored per-board.**

  * Each board is completely isolated in its own directory: `db.sqlite` for posts and an `uploads/` folder for attachments.
* **The landing page is a static HTML file you edit by hand.**

  * Add or remove board links yourself for total control over what is visible to users.
* **No web-based admin panel required.**

  * All board management is via file manager or SFTP. There is nothing to hack and no risk of web-based admin mistakes.

---

## How is Chessboard Different?

* **Maximum Security and Auditability**

  * No complex web admin UIs or dynamic board creation by users.
  * If you don’t want a board, just delete its directory. No orphaned data, no lingering files.
  * Backups and restores are trivial: copy the folder, that’s it.
* **Focused on Education and Organized Use**

  * Designed for clarity and maintainability, not for anonymous entertainment or high-volume chat.
  * Teachers, mentors, and serious hobbyists can keep control over every aspect of the discussion space.
* **Consistent and Maintainable**

  * The entire site uses one set of templates and code.
  * If you update a template or backend logic, every board is instantly updated. No migrations, no per-board upgrades, no risk of stale logic.
* **No Surprises, No Bloat**

  * You control which boards exist, and what appears on the home page. No features will suddenly appear or break.
  * No bots or users can create, rename, or delete boards—only the server owner can do this.
* **Content is Always Yours**

  * Everything is on your server, in human-readable and easily managed files. No cloud lock-in, no company storing your users’ data.

---

## Benefits for Educators and Knowledge-Focused Groups

* **Reliability:** If you can copy a folder, you can backup or restore your board. No lost content, no maintenance windows.
* **Transparency:** You always know what’s running. No hidden tables, no secret features.
* **Privacy:** No advertising, no trackers, no required email addresses.
* **Adaptability:** Want to change how every board looks? Edit one template. Done. Want to try new file upload rules, or add post length limits? One change affects all boards at once.
* **Audit-Ready:** You can see every board, every file, and every post on disk.

---

## Typical Usage Example

1. **Edit your landing page** (`static/index.html`) to link to boards (e.g. `/chess/logic101/`, `/chess/calculus/`).
2. **Create a board:**

   * Create a folder under `/chess/` with your desired board name.
   * (The app will create the database and uploads folder on first visit if they don’t exist.)
3. **Delete a board:**

   * Remove the corresponding folder under `/chess/`.
   * Remove the link from the landing page.
4. **Back up a board:**

   * Just copy its folder.

---

## Why Not Use Entertainment-Focused Boards?

* **Entertainment boards** are optimized for maximum engagement, anonymous posting, rapid thread turnover, and meme culture. Their security, privacy, and admin controls are often an afterthought.
* **Chessboard** is built for *discussion that matters*. It’s designed to:

  * Be controlled by educators and admins, not by users or bots
  * Keep all content reviewable, backup-able, and never “lost” in a massive database
  * Stay online for years with zero maintenance headaches
  * Prevent accidental exposure or destruction of educational content

---

## Technology Stack

* **Rust**: Speed, safety, and no garbage collector pauses
* **Actix-web**: Modern, secure web framework
* **Askama**: Type-safe, fast HTML templates
* **SQLite**: Each board’s data is isolated and robust
* **Zero dependencies on the cloud**

---

## FAQ

**Q: Can anyone create or delete boards from the web?**

* No. Only the site admin (with file access) can create or remove boards.

**Q: Can I customize the look or add new features?**

* Yes! All boards use the same code and templates. Edit them and restart, and all boards update at once.

**Q: Can I lose data?**

* Only if you delete a board’s folder or your disk fails. Backups are easy—just copy the folder.

**Q: What if a board gets spammed?**

* Since all content is on disk, moderation and post deletion tools are easy to build or operate by hand. No content is ever “hidden.”

---

## Final Thoughts

**Chessboard** is for teachers, clubs, small teams, and anyone who wants a message board that is safe, reliable, transparent, and fully under their control—without the headaches, security risks, or bloat of entertainment boards.

If you care about knowledge and clarity, and want to *own* your discussion platform, this is for you.
