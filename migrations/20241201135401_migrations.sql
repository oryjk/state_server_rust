-- Add migration script here
create table ss_client_state_log
(
    id         INT AUTO_INCREMENT PRIMARY KEY,
    client_id  VARCHAR(255) NOT NULL,
    status     TEXT         NOT NULL,
    created_at TIMESTAMP    NOT NULL
)