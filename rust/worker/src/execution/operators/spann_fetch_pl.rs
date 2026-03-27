use async_trait::async_trait;
use chroma_error::{ChromaError, ErrorCodes};
use chroma_index::spann::types::SpannPosting;
use chroma_segment::distributed_spann::{SpannSegmentReader, SpannSegmentReaderError};
use chroma_system::{Operator, OperatorType};
use chroma_types::{CollectionUuid, SegmentUuid};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct SpannFetchPlInput<'referred_data> {
    pub(crate) reader: Option<SpannSegmentReader<'referred_data>>,
    pub(crate) head_id: u32,
}

#[derive(Debug)]
pub(crate) struct SpannFetchPlOutput {
    pub(crate) posting_list: Vec<SpannPosting>,
}

#[derive(Error, Debug)]
pub enum SpannFetchPlError {
    #[error("Error creating spann segment reader for head_id={head_id}")]
    SpannSegmentReaderCreationError { head_id: u32 },
    #[error(
        "Error querying spann reader for collection_id={collection_id}, segment_id={segment_id}, head_id={head_id}: {source}"
    )]
    SpannSegmentReaderError {
        collection_id: CollectionUuid,
        segment_id: SegmentUuid,
        head_id: u32,
        #[source]
        source: SpannSegmentReaderError,
    },
}

impl ChromaError for SpannFetchPlError {
    fn code(&self) -> ErrorCodes {
        match self {
            Self::SpannSegmentReaderCreationError { .. } => ErrorCodes::Internal,
            Self::SpannSegmentReaderError { source, .. } => source.code(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SpannFetchPlOperator {}

impl SpannFetchPlOperator {
    #[allow(dead_code)]
    pub fn new() -> Box<Self> {
        Box::new(SpannFetchPlOperator {})
    }
}

#[async_trait]
impl Operator<SpannFetchPlInput<'_>, SpannFetchPlOutput> for SpannFetchPlOperator {
    type Error = SpannFetchPlError;

    async fn run(
        &self,
        input: &SpannFetchPlInput,
    ) -> Result<SpannFetchPlOutput, SpannFetchPlError> {
        match &input.reader {
            Some(reader) => {
                let posting_list =
                    reader
                        .fetch_posting_list(input.head_id)
                        .await
                        .map_err(|source| SpannFetchPlError::SpannSegmentReaderError {
                            collection_id: reader.collection_id(),
                            segment_id: reader.segment_id(),
                            head_id: input.head_id,
                            source,
                        })?;
                Ok(SpannFetchPlOutput { posting_list })
            }
            None => {
                return Err(SpannFetchPlError::SpannSegmentReaderCreationError {
                    head_id: input.head_id,
                });
            }
        }
    }

    // This operator is IO bound.
    fn get_type(&self) -> OperatorType {
        OperatorType::IO
    }
}
