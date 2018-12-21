ALTER TABLE users
ADD CONSTRAINT users_referal_fkey
   FOREIGN KEY (referal)
   REFERENCES users(id);
