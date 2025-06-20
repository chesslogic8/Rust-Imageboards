


# imageboard3 - takes things to a whole new level. 


# Chessboard: A Simple, Secure, Self-Hosted Message Board

## What is Chessboard?

Chessboard is a lightweight, modern, and rock-solid imageboard/messageboard platform designed for hobbyists, clubs, or private communities. It puts you—the admin—fully in control, while making life easy for both users and maintainers.

## Key Benefits

### 1. **Simplicity and Full Control**

* **No database admin tools, no migrations, no surprises.**
* You manage boards by simply adding or deleting directories on your server.
* The main landing page is just a static HTML file you can edit and upload by hand.
* No web admin UI is required for basic operation.

### 2. **Ultra-Reliable and Safe**

* Every board’s content (posts and uploads) is stored in its own folder—no cross-contamination or risk of corrupting other boards.
* Deleting a board is as easy as deleting its directory. No orphaned files or entries, ever.
* Restoring or backing up is just copying a directory.
* No dynamic homepage code = zero chance of accidental exposure or data leaks.

### 3. **Instant Feature Updates for All Boards**

* All boards use the same code and templates. If you update a feature or fix a bug, it’s fixed for every board, instantly—no migrations or downtime.
* You can update the user experience, validation, security rules, or styling for all boards in one place.

### 4. **No Bloat, No Automation Gone Wrong**

* No user-facing board creation or deletion features (unless you want to add them). Only people with server/SFTP access can change the board structure.
* No automated scripts will add, expose, or remove boards you didn’t intend.

### 5. **Maximum Transparency**

* At any moment, you can see all content just by looking at the `chess/` directory and your static `index.html`.
* You know exactly what’s running, what’s live, and how to back it up.

### 6. **Easy Backups and Portability**

* Zip a directory or copy/paste to move or clone a board.
* If you ever want to move the site or individual boards, just move folders and static files.

### 7. **Powerful, Modern Rust Stack**

* All security features, validation, and template rendering are handled with up-to-date best practices using Actix-web, r2d2\_sqlite, and Askama.
* Safe file upload handling with strict allow-lists and size checks.
* Board content can be rendered beautifully and safely thanks to Rust’s memory safety and Askama’s robust escaping.

## How is Chessboard Different?

* **Most forum/imageboard software:**

  * Requires complex web UIs and admin backends.
  * Stores all boards in a single monolithic database.
  * Dynamic board lists that can’t be easily audited.
  * Difficult to safely delete boards without lingering files or data.

* **Chessboard:**

  * Each board is just a directory. No hidden state. No complex migrations.
  * Admin work = file manager/SFTP, not a web UI or SQL.
  * Static, hand-edited landing page means you decide what users see. No risk of accidental leaks.
  * If you need to delete or back up a board, it’s a single directory operation.
  * If you want to extend features, you do so in one place, instantly improving every board.

## How Does It Work?

* You edit your homepage (`static/index.html`) with the board links you want.
* To create a board: create a folder in `/chess/boardname` (no need for a DB or uploads folder; the app will create those on first visit).
* To delete a board: remove that folder and its content is instantly and safely gone.
* Users visit `/chess/boardname/` and get the board view, post threads, and upload files—using the same experience on every board.

## Who Is It For?

* Small forums, clubs, or teams who want absolute control.
* Anyone who cares about privacy, simplicity, and reliability.
* Admins who prefer SFTP/file management to risky web admin panels.
* Anyone who wants a truly transparent, hackable, and robust board.

---

**Questions or want to add more features? All boards get better together!**
