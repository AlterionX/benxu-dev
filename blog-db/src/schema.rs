table! {
    google_sso (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
        user_id -> Uuid,
    }
}

table! {
    passwords (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
        user_id -> Uuid,
        hash -> Text,
    }
}

table! {
    posts (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        created_by -> Uuid,
        updated_at -> Timestamptz,
        updated_by -> Uuid,
        published_at -> Nullable<Timestamptz>,
        published_by -> Nullable<Uuid>,
        archived_at -> Nullable<Timestamptz>,
        archived_by -> Nullable<Uuid>,
        deleted_at -> Nullable<Timestamptz>,
        deleted_by -> Nullable<Uuid>,
        title -> Text,
        body -> Text,
    }
}

table! {
    post_tag_junctions (post_id, tag_id) {
        post_id -> Uuid,
        tag_id -> Uuid,
        created_at -> Timestamptz,
        created_by -> Uuid,
    }
}

table! {
    tags (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        created_by -> Uuid,
        name -> Text,
        description -> Text,
    }
}

table! {
    users (id) {
        id -> Uuid,
        user_name -> Nullable<Text>,
        created_at -> Timestamptz,
        created_by -> Nullable<Uuid>,
        updated_at -> Timestamptz,
        updated_by -> Nullable<Uuid>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        email -> Nullable<Varchar>,
    }
}

joinable!(post_tag_junctions -> posts (post_id));
joinable!(post_tag_junctions -> tags (tag_id));
joinable!(post_tag_junctions -> users (created_by));
joinable!(tags -> users (created_by));

allow_tables_to_appear_in_same_query!(
    google_sso,
    passwords,
    posts,
    post_tag_junctions,
    tags,
    users,
);
