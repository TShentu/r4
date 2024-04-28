use std::{collections::HashMap, env};

use anyhow::{Error as AnyhowError, Result as AnyhowResult};
use candle_core::{Device, Tensor};
use candle_transformers::models::bert::BertModel;
use log::{debug as logdebug, error as logerror};
use ndarray_rand::rand;
use rand::seq::SliceRandom;

use crate::{
    embedding_common::{self, normalize_l2, TokenizerImplSimple},
    knowledge_base_api,
};

const HUGGING_FACE_MODEL_NAME: &str = "sentence-transformers/all-MiniLM-L6-v2";
const HUGGING_FACE_MODEL_REVISION: &str = "refs/pr/21";
pub const BERTV2_EMBEDDING_DIMENSION: usize = 384;

pub async fn calculate_single_entry_pure(
    current_entry_id: &String,
    model: &BertModel,
    current_tokenizer: &embedding_common::TokenizerImplSimple,
) -> Option<Tensor> {
    let current_entry_option = knowledge_base_api::get_entry_by_id(current_entry_id).await;
    if let Some(current_entry) = current_entry_option {
        if let Some(current_title) = current_entry.titile {
            let current_result: Result<Tensor, AnyhowError> =
                embedding_common::calculate_one_sentence(
                    &model,
                    current_tokenizer,
                    current_title,
                    512,
                );
            match current_result {
                Ok(current_tensor) => {
                    let last_tensor_result = current_tensor.get(0);
                    match last_tensor_result {
                        Ok(current_last_tensor) => {
                            return Some(current_last_tensor);
                        }
                        Err(err) => {
                            logerror!(
                                "get entry {} embedding zero dimension error {}",
                                current_entry_id,
                                err.to_string()
                            );

                            return None;
                        }
                    }
                }
                Err(err) => {
                    logerror!(
                        "calculate entry {} embedding error {}",
                        current_entry_id,
                        err.to_string()
                    );
                    return None;
                }
            }
        } else {
            logerror!("entry {} have not title", current_entry_id);
            return None;
        }
    } else {
        logerror!("entry {} not exist", current_entry_id);
        return None;
    }
}

async fn calculate_userembedding() -> AnyhowResult<Tensor, AnyhowError> {
    let current_source_name: String = env::var("TERMINUS_RECOMMEND_SOURCE_NAME")
        .expect("TERMINUS_RECOMMEND_SOURCE_NAME env not found.");

    let cumulative_embedding_data: [f32; BERTV2_EMBEDDING_DIMENSION] =
        [0f32; BERTV2_EMBEDDING_DIMENSION];
    let mut cumulative_tensor = Tensor::new(&cumulative_embedding_data, &Device::Cpu)?;

    let default_model = HUGGING_FACE_MODEL_NAME.to_string();
    let default_revision = HUGGING_FACE_MODEL_REVISION.to_string();
    let (model, mut tokenizer) =
        embedding_common::build_model_and_tokenizer(default_model, default_revision).unwrap();
    let current_tokenizer: &TokenizerImplSimple = tokenizer
        .with_padding(None)
        .with_truncation(None)
        .map_err(AnyhowError::msg)
        .unwrap();

    let impression_id_to_entry_id: HashMap<String, String> =
        embedding_common::retrieve_wise_library_impression_knowledge().await?;
    let mut wise_library_entry_ids: Vec<String> = Vec::new();
    for (_, current_entry_id) in impression_id_to_entry_id {
        wise_library_entry_ids.push(current_entry_id)
    }
    let one_hundred_batch_wise_library: Vec<&String> = wise_library_entry_ids
        .choose_multiple(&mut rand::thread_rng(), 100)
        .collect();
    for current_entry_id in one_hundred_batch_wise_library {
        let current_tensor_option =
            calculate_single_entry_pure(current_entry_id, &model, current_tokenizer).await;
        if let Some(current_tensor) = current_tensor_option {
            cumulative_tensor = cumulative_tensor.add(&current_tensor)?;
            logdebug!("add current_entry {}", current_entry_id);
        } else {
            logerror!("current_entry_id {} calculate fail", current_entry_id);
        }
    }

    let current_algorithm_tensor_option =
        embedding_common::retrieve_current_algorithm_impression_knowledge(
            current_source_name.clone(),
            BERTV2_EMBEDDING_DIMENSION,
        )
        .await;
    if let Some(current_algorithm_tensor) = current_algorithm_tensor_option {
        logdebug!(
            "current algorithm existing embedding cumulative result {:?}",
            current_algorithm_tensor.to_vec1::<f32>().unwrap()
        );
        cumulative_tensor = cumulative_tensor.add(&current_algorithm_tensor)?;
    } else {
        logerror!(
            "retrieve source {} embedding tensor fail ",
            current_source_name.clone()
        );
    }

    cumulative_tensor = normalize_l2(&cumulative_tensor, 0)?;
    logdebug!(
        "cumulative_tensor {:?}",
        cumulative_tensor.to_vec1::<f32>().unwrap()
    );
    Ok(cumulative_tensor)
}

pub async fn execute_bertv2_user_embedding() {
    let user_embedding: Tensor = calculate_userembedding()
        .await
        .expect("calculate user embedding fail");
    let original_user_embedding =
        embedding_common::retrieve_user_embedding_through_knowledge(BERTV2_EMBEDDING_DIMENSION)
            .await
            .expect("retrieve user embedding through knowledge base fail");
    let new_user_embedding_result = user_embedding.add(&original_user_embedding);
    match new_user_embedding_result {
        Ok(current_new_user_embedding) => {
            let normalized_new_user_embedding = normalize_l2(&current_new_user_embedding, 0)
                .expect("normalize new user embedding fail");
            embedding_common::set_user_embedding_knowledgebase(&normalized_new_user_embedding)
                .await;
            logdebug!("set success for new user embedding");
        }
        Err(err) => {
            logerror!("old and new user embedding add fail {},", err.to_string())
        }
    }
}

mod bertv2test {
    use std::env;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::common;
    use crate::common_test_operation;

    #[tokio::test]
    async fn test_calculate_single_entry() {
        // cargo test bertv2test::test_calculate_single_entry
        common_test_operation::init_env();
        let default_model = HUGGING_FACE_MODEL_NAME.to_string();
        let default_revision = HUGGING_FACE_MODEL_REVISION.to_string();
        let (model, mut tokenizer) =
            embedding_common::build_model_and_tokenizer(default_model, default_revision).unwrap();
        let current_tokenizer: &TokenizerImplSimple = tokenizer
            .with_padding(None)
            .with_truncation(None)
            .map_err(AnyhowError::msg)
            .unwrap();
        let current_entry_id: String = "6555bef91d141d0bf00224ec".to_string();
        let current_tesnor =
            calculate_single_entry_pure(&current_entry_id, &model, current_tokenizer)
                .await
                .expect("get tensor fail");
        logdebug!(
            "current_tensor***************************** {}",
            current_tesnor
        )
    }

    #[tokio::test]
    async fn test_set_init_user_embedding() {
        common_test_operation::init_env();
        let init_embedding = embedding_common::init_user_embedding(BERTV2_EMBEDDING_DIMENSION);
        embedding_common::set_user_embedding_knowledgebase(&init_embedding).await;
    }
}
