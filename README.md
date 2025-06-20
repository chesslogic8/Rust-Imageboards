




# Rust is god. 

Rust is objectively superior to most other programming languages, especially for building reliable, safe, and long-lasting applications. Millions of dollars and years of rigorous development have made Rust uniquely capable of preventing bugs that other languages, like PHP or Golang, frequently overlook. This isn't just an opinion; it is a demonstrable fact.



#  Imageboard menu of starter apps 

# Kept in early dev stages so you can instantly start dev from a working imageboard app! Do not know how to code in Rust? NO PROBLEM!! Ai can write your code! 

Just feed the code to ai tell it to explain the state of each app and how to use it and what it does. The apps are minimal, but they all function. 

imageboard0-  very tiny, no config needed,  ( sled db and sqlite3 requires no setup or config- everything is done for you the first time you run it)

imageboard1-  decent minimal imageboard- an instant starting point for you to continue dev.

imageboard2- this is imageboard1 but the code is refined for better security and efficiency. 

imageboard3- First generation of a different take on ib.  Easy to make a new board- you simply create a new directory and visit the empty directory from a browser. To delete a board you simply delete the entire board directory. To back up your site simply back up the board directories. Rock solid, simple, secure.

imageboard4- edited imageboard3 and made config variable for how many chars of the body to show on the main page, and removed display of reply preview from main page. 

imageboard5- this is imageboard4 with headless admin implemented. Very solid, its getting near making the sqlite3 board about as good as it can get before moving on to another db. 

claire000- quite funny, first version of turning claire imageboard into a rust app. Needs a lot of work, off to a great start tho! 

imageboard6- made to look similar to tiny ib. not complete, this is just the  first ver. 

Imageboard7- more features to imageboard6. Switched to maria db. Requires a specific db /password to exist. Just ask ai if you do not know how to set up the right db to work with this. 

imageboard8- better version of imageboard 7. Defines the db in .env file.  pagination, reply to threads, nice looking. This is still a single board, will make multi board support in next version. Why post this then? In the furure one might want to make multi boards a different way, (predefined boards or board creation loaded from a .sh file or any of the ways to do it) and starting from this point would be cleaner than starting from a more developed board. 

imageboard9- better version of imageboard 8- multi board enabled
















