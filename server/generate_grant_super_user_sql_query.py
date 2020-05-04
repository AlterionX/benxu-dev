#!python

import uuid

associated_user_id = "835b5bc7-6117-4669-9202-1d4acd7ad204"

capabilities = [
    "edit_post",
    "create_post",
    "delete_post",
    "publish_post",
    "archive_post",
    "create_user",
    "edit_user",
    "delete_user",
    "grant_permission",
    "view_permission",
    "edit_user_credentials",
    "delete_permission"
]

query_records = [fr"('{uuid.uuid4()}', '{associated_user_id}', '{associated_user_id}', '{cap}')" for cap in capabilities]

sql_cmd_head = r"""INSERT INTO permissions (
    id,
    created_by,
    user_id,
    permission
) VALUES
"""
sql_cmd_tail = ",\n".join(query_records) + ";"

print(sql_cmd_head, sql_cmd_tail)
