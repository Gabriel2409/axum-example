-- DEV ONLY - First we get all the pid of the process that correspond to the db connection 
-- we want to terminate and call pg_terminate_backend to terminate it

-- it is important to not use default db and username because it can be a pain to refactor
-- later otherwise
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE
 usename = 'app_user' OR datname = 'app_db';

-- DEV ONLY - Brute Force DROP DB (for local dev and unit test)
DROP DATABASE IF EXISTS app_db;
DROP USER IF EXISTS app_user;

-- DEV ONLY - Dev only password (for local dev and unit test).
CREATE USER app_user PASSWORD 'dev_only_pwd';
CREATE DATABASE app_db owner app_user ENCODING = 'UTF-8';
