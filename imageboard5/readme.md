# Chessboard – A Minimal, Reliable, Moderated Message Board

Chessboard is a self-hosted, file-based, ultra-reliable message board and imageboard system.  
**Designed for educators, teams, and focused communities**, it puts you—the admin—in total control.

---

## Features

- **Boards are directories.**  
  To create a board, make a folder under `/chess/`.  
  To remove a board, delete the folder.  
  Each board is independent and easy to back up.

- **Static, admin-edited homepage.**  
  The landing page (`static/index.html`) is static—edit it by hand to add or remove board links.

- **Modern, secure Rust backend.**  
  Uses Actix-web, Askama, and SQLite for safety, performance, and maintainability.

- **File uploads for threads.**  
  Attach images or videos (max 50MB each; configurable) to any new thread.

- **Thread and reply support.**  
  Threads can have unlimited replies.  
  Replies are always visible, with optional image/video in the thread’s first post.

- **Ultra-simple moderation:**  
  - In every thread, a small `[x]` button appears at the bottom right of each post and reply.
  - **Click `[x]` to delete:** Enter your admin password to confirm.
  - Deleting a thread removes all replies and attached images.
  - Deleting a reply removes just that reply.
  - No need for a special “admin mode”—the `[x]` is always visible but protected by password.
  - All actions are transactional and safe; images are deleted from disk as needed.

- **No web-based registration or account system.**  
  All moderation is done with a single strong admin password.

- **No placeholders.**  
  Deleted posts vanish immediately; the board stays uncluttered.

- **Disaster-proof:**  
  - Back up any board by copying its directory.
  - Restore instantly by copying a folder back.
  - If a board is spammed, simply restore from backup.

---

## Why is Chessboard Different?

- **Built for reliability, not entertainment.**  
  - No memes, no anonymous chaos, no accidental mass deletions.
  - Ideal for teachers, club leaders, and anyone who values clarity over novelty.

- **Minimal attack surface.**  
  - No dynamic “admin panel.”  
  - No login/session management code to break.

- **Transparent and auditable.**  
  - All content is visible on disk; nothing is hidden in a monolithic database.
  - You always know exactly what exists and who can delete it.

- **Simple, not simplistic.**  
  - All the power you need—none of the distractions you don’t.

---

## Usage

### Create a Board

1. Create a new directory under `chess/`, e.g. `chess/kingsgambit/`.
2. Add a link to this board in your `static/index.html`.

### Remove a Board

1. Delete the corresponding folder under `chess/`.
2. Remove its link from `static/index.html`.

### Moderate Posts or Replies

1. Visit any thread (e.g. `/chess/kingsgambit/thread/1`).
2. Click `[x]` at the bottom right of the post or reply you want to delete.
3. Enter your admin password and confirm.
4. The post (and, if it’s the original post, all its replies and attachments) are deleted immediately.

### Backup & Restore

- Copy any board’s directory (`chess/{board}`) to back it up.
- Restore by copying the backup folder back into `chess/`.

---

## Admin Notes

- **Change your admin password!**  
  In `src/main.rs`, set your own strong `ADMIN_PASSWORD` at the top.

- **No email, usernames, or registration needed.**  
  Only the admin password is required for moderation.

- **All deletes are permanent.**  
  Make regular backups if you want an undo option.

- **Posts and replies are stored in per-board SQLite databases.**  
  This design makes disaster recovery and migration easy.

---

## Philosophy

Chessboard is not for “maximum engagement.”  
It is for **maximum clarity, safety, and focus**—a modern, hack-resistant take on classic messageboards, ready to serve educational or specialized discussion for years with minimal maintenance.

---

**Questions or need to extend the system? Just ask!**
