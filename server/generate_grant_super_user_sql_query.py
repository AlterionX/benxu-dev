#!python

inserts = [
    ("\'9274e77a-e247-4a4b-a943-0b1b7215c59e\'", "\'edit_post\'"),
    ("\'475793d3-7eb4-40a8-8b0b-bc57df8e7ee6\'", "\'create_post\'"),
    ("\'e2d546f7-b934-4eb2-9e01-7d9025a145bc\'", "\'delete_post\'"),
    ("\'779b8885-4d30-46b9-b6b3-16178fdcf45d\'", "\'publish_post\'"),
    ("\'f0c7b140-262d-4bb6-8aac-f9d26157d9da\'", "\'archive_post\'"),
    ("\'b49b8b9f-723f-41e9-b4ab-5d76b6c05c0b\'", "\'create_user\'"),
    ("\'6a13e444-d60f-4572-b38f-1cb97dd0041c\'", "\'edit_user\'"),
    ("\'94ed46b9-755d-4776-8c2e-64e9dc8f7f01\'", "\'delete_user\'"),
    ("\'7522f59b-97a8-4d89-baab-f37547849e18\'", "\'grant_permission\'"),
    ("\'80db4b56-8699-4f26-86e8-4702126b69b5\'", "\'view_permission\'"),
    ("\'fad43fd9-6722-422e-bd3f-1ec7371d8058\'", "\'edit_user_credentials\'"),
    ("\'85101413-8494-4bcd-97fa-4725807433d3\'", "\'delete_permission\'")
]

print("""INSERT INTO permissions (
    id,
    created_by,
    user_id,
    permission
) VALUES""")
for uuid, perm in inserts:
    print("("
        + uuid + ", "
        + "\'2ec16940-b926-4eec-9b72-fbdb3c4d75f2\', "
        + "\'2ec16940-b926-4eec-9b72-fbdb3c4d75f2\', "
        + perm
        + "),"
    )
