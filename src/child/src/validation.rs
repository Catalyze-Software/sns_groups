use ic_scalable_misc::{
    enums::{api_error_type::ApiError, validation_type::ValidationType},
    helpers::validation_helper::Validator,
    models::validation_models::ValidateField,
};

use shared::group_model::{PostGroup, UpdateGroup};

pub fn validate_post_group(post_group: PostGroup) -> Result<(), ApiError> {
    let validator_fields = vec![
        ValidateField(
            ValidationType::StringLength(post_group.name, 3, 64),
            "name".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(post_group.description, 0, 2500),
            "description".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(post_group.website, 0, 200),
            "website".to_string(),
        ),
        ValidateField(
            ValidationType::Count(post_group.tags.len(), 0, 25),
            "tags".to_string(),
        ),
    ];

    Validator(validator_fields).validate()
}

pub fn validate_update_group(update_group: UpdateGroup) -> Result<(), ApiError> {
    let validator_fields = vec![
        ValidateField(
            ValidationType::StringLength(update_group.name, 3, 64),
            "name".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(update_group.description, 0, 2500),
            "description".to_string(),
        ),
        ValidateField(
            ValidationType::StringLength(update_group.website, 0, 200),
            "website".to_string(),
        ),
        ValidateField(
            ValidationType::Count(update_group.tags.len(), 0, 25),
            "tags".to_string(),
        ),
    ];

    Validator(validator_fields).validate()
}
