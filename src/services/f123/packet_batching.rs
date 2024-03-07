use std::{cell::Cell, sync::Arc};

use crate::{
    cache::F123InsiderCache,
    config::constants::BATCHING_INTERVAL,
    error::{AppResult, F123ServiceError},
    protos::{batched::ToProtoMessageBatched, packet_header::PacketType, PacketHeader},
    structs::OptionalMessage,
};
use ntex::util::Bytes;
use parking_lot::Mutex;
use tokio::{
    sync::{broadcast::Sender, oneshot},
    time::interval,
};
use tracing::{info, warn};

const BATCHING_VECTOR_CAPACITY: usize = 2048;

/// `PacketBatching` is responsible for collecting packets over a period of 700ms,
/// batching them together, and then sending a single batched packet. It serves
/// as a buffering layer that aggregates packets to minimize processing and sending overhead.
///
/// In addition to batching packets, it also forwards all received packets to a cache for
/// potential further use or analysis.
///
/// # Fields
///
/// - `buf`: An `Arc<Mutex<Vec<PacketHeader>>>` that temporarily stores incoming packet headers
///   before they are batched. This buffer accumulates packets over the 700ms window.
///
/// - `shutdown`: An `Option<oneshot::Sender<()>>` used to signal the shutdown of the packet
///   batching process. Once a shutdown signal is received, no more packets are accepted,
///   and the current batch is processed and sent.
///
/// - `cache`: `F123InsiderCache` is used to store packets as they arrive. This allows for
///   caching of individual packets alongside the batching process, enabling both immediate
///   and batched packet processing.
///
/// # Functionality
///
/// These structs primary function is to optimize packet processing by reducing the number
/// of individual packets that need to be handled. By batching packets together and caching
/// them simultaneously, it enables efficient packet management and processing, particularly
/// in high-throughput scenarios.

pub struct PacketBatching {
    buf: Arc<Mutex<Vec<PacketHeader>>>,
    shutdown: Option<oneshot::Sender<()>>,
    cache: F123InsiderCache,
}

impl PacketBatching {
    /// Creates a new `PacketBatching` instance with a sender for batched data and a cache.
    ///
    /// This function sets up a new `PacketBatching` with an internal buffer for packet headers,
    /// a shutdown mechanism, and a reference to a cache for additional data handling. It also spawns
    /// a background task that periodically sends data from the buffer to the specified sender every 700ms,
    /// ensuring that any collected packets are batched and sent in a single operation. This task listens
    /// for a shutdown signal to gracefully stop its operation.
    ///
    /// # Parameters
    ///
    /// - `tx`: A `Sender<Bytes>` used to send the batched data to a receiver.
    /// - `cache`: `F123InsiderCache`, a cache used for storing or further processing of packets.
    ///
    /// # Returns
    ///
    /// Returns an instance of `PacketBatching` ready to receive packets and handle batching operations.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let (tx, rx) = tokio::sync::mpsc::channel(100);
    /// let cache = F123InsiderCache::new(); // Assuming F123InsiderCache::new is defined
    /// let packet_batches = PacketBatching::new(tx, cache);
    /// ```
    ///
    /// # Note
    ///
    /// The background task for sending batched data operates until a shutdown signal is received.
    /// Ensure to send a shutdown signal through the `shutdown` channel when the `PacketBatching`
    /// instance is no longer needed to properly clean up resources.
    pub fn new(tx: Sender<Bytes>, cache: F123InsiderCache) -> Self {
        let (otx, orx) = oneshot::channel::<()>();
        let buf = Arc::from(Mutex::from(Vec::with_capacity(BATCHING_VECTOR_CAPACITY)));

        let instance = Self {
            buf: buf.clone(),
            shutdown: Some(otx),
            cache,
        };

        let mut orx = Cell::from(orx);
        let mut interval_timer = interval(BATCHING_INTERVAL);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        if let Err(e) = Self::send_data(&buf, &tx).await {
                            warn!("Packet batching: {}", e);
                        }
                    }

                    _ = orx.get_mut() => {
                        break;
                    }
                }
            }
        });

        instance
    }

    /// Asynchronously pushes a `PacketHeader` into the internal buffer and caches it.
    ///
    /// This method is a convenience wrapper around `push_with_optional_parameter`, implicitly
    /// passing `None` for the optional parameter. It encodes the given packet and stores it
    /// both in an internal buffer and a cache, based on the packet type.
    ///
    /// # Parameters
    ///
    /// - `packet`: The `PacketHeader` to be pushed and cached.
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<()>` indicating the success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut batching = PacketBatching::new(); // Assuming a constructor for PacketBatching
    /// let packet = PacketHeader::new(); // Assuming a constructor for PacketHeader
    /// batching.push(packet).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function returns an error if encoding the packet or caching it fails.
    #[inline(always)]
    pub async fn push(&mut self, packet: PacketHeader) -> AppResult<()> {
        self.push_with_optional_parameter(packet, None).await
    }

    /// Asynchronously pushes a `PacketHeader` into the internal buffer with an optional parameter,
    /// and caches it.
    ///
    /// Encodes the given packet, stores it in the internal buffer, and caches it according to its
    /// packet type. If provided, the optional parameter is used for additional processing or caching
    /// logic specific to certain packet types.
    ///
    /// # Parameters
    ///
    /// - `packet`: The `PacketHeader` to be pushed and cached.
    /// - `second_param`: An optional `OptionalMessage` used for certain packet types that require
    ///   additional data.
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<()>` indicating the success or failure of the operation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut batching = PacketBatching::new(); // Assuming a constructor for PacketBatching
    /// let packet = PacketHeader::new(); // Assuming a constructor for PacketHeader
    /// let optional_message = Some(OptionalMessage::Text("Example text")); // Example of an optional parameter
    /// batching.push_with_optional_parameter(packet, optional_message).await?;
    /// ```
    ///
    /// # Errors
    ///
    /// This function returns an error if encoding the packet, determining its type, or caching it fails.
    pub async fn push_with_optional_parameter(
        &mut self,
        packet: PacketHeader,
        second_param: Option<OptionalMessage<'_>>,
    ) -> AppResult<()> {
        let packet_type = packet.r#type();

        self.save_cache(packet_type, &packet.payload, second_param)
            .await?;

        self.buf.lock().push(packet);
        Ok(())
    }

    /// Sends batched packet headers from the buffer to the specified sender asynchronously.
    ///
    /// This function locks the shared buffer, drains it if not empty, and attempts to batch encode
    /// the packet headers. If successful, the batch is compressed and sent to the receiver if there
    /// are any listeners. The operation might fail if encoding the batched data or sending it through
    /// the channel encounters errors.
    ///
    /// # Parameters
    ///
    /// - `buf`: A reference to an `Arc<Mutex<Vec<PacketHeader>>>` representing the shared buffer of packet headers.
    /// - `tx`: A reference to a `Sender<Bytes>` used to send the encoded and compressed batch of packet headers.
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` indicating the success or failure of the operation.
    ///
    /// # Errors
    ///
    /// - Returns `Ok(())` immediately if the buffer is empty, indicating there's nothing to send.
    /// - Returns an error if batch encoding fails or if sending the encoded batch through the channel fails.
    ///
    /// # Usage
    ///
    /// This function is designed to be called exclusively from the background thread initiated by
    /// `PacketBatching::new`. Calling this function from other contexts may result in unexpected behavior
    /// or race conditions due to its reliance on shared state and specific timing.
    ///
    /// # Examples
    ///
    /// Assuming `buf` and `tx` have been properly initialized and a `PacketBatching` instance is managing the call:
    ///
    /// ```ignore
    /// tokio::spawn(async {
    ///     if let Err(e) = send_data(&buf, &tx).await {
    ///         warn!("Error sending batched data: {}", e);
    ///     }
    /// });
    /// ```
    ///
    /// Note: This example assumes that `buf` and `tx` are appropriately set up and that the asynchronous
    /// context is handled by the caller (e.g., a tokio runtime).
    async fn send_data(buf: &Arc<Mutex<Vec<PacketHeader>>>, tx: &Sender<Bytes>) -> AppResult<()> {
        let buf = {
            let mut buf = buf.lock();

            if buf.is_empty() {
                return Ok(());
            }

            let mut taken_buf = Vec::with_capacity(BATCHING_VECTOR_CAPACITY);
            std::mem::swap(&mut taken_buf, &mut *buf);

            taken_buf
        };

        // TODO: Implement another cache method for events
        if let Some(batch) = ToProtoMessageBatched::batched_encoded(buf) {
            let encoded_batch = Self::compress(&batch)?;

            if tx.receiver_count() == 0 {
                return Ok(());
            }

            if let Err(e) = tx.send(encoded_batch) {
                warn!("Broadcast channel: {}", e);
            };
        } else {
            Err(F123ServiceError::BatchedEncoding)?
        }

        Ok(())
    }

    /// Asynchronously saves encoded packet data to the cache based on the packet type.
    ///
    /// This method updates the cache with the provided `encoded_package` depending on the
    /// `packet_type`. For `SessionHistoryData` and `EventData`, it uses the `second_param`
    /// to further refine the caching operation. This method logs a warning if any cache
    /// update operation fails.
    ///
    /// # Parameters
    ///
    /// - `packet_type`: The type of packet being saved, determining how the data is processed
    ///   and stored in the cache.
    /// - `encoded_package`: A slice of bytes representing the encoded packet data to be stored.
    /// - `second_param`: An optional parameter used for `SessionHistoryData` (providing the car ID)
    ///   and `EventData` (providing the event string code). It is ignored for other packet types.
    ///
    /// # Returns
    ///
    /// Returns an `AppResult<()>` indicating the success or failure of the cache update operation.
    /// In case of success, it returns `Ok(())`. Errors during cache update operations are logged
    /// as warnings and do not affect the return value.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use your_crate::{YourStruct, PacketType, OptionalMessage, AppResult};
    /// # async fn example(mut instance: YourStruct) -> AppResult<()> {
    /// let packet_type = PacketType::CarMotion;
    /// let encoded_package = &[1, 2, 3]; // Example encoded data
    /// let second_param = None; // Not used for this packet type
    /// instance.save_cache(packet_type, encoded_package, second_param).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This function logs errors instead of returning them. It always returns `Ok(())` unless
    /// an unrecoverable error occurs during operations not directly related to cache updating,
    /// such as a failure in pruning the cache for `FinalClassificationData`.
    #[inline(always)]
    async fn save_cache(
        &mut self,
        packet_type: PacketType,
        encoded_package: &[u8],
        second_param: Option<OptionalMessage<'_>>,
    ) -> AppResult<()> {
        match packet_type {
            PacketType::CarMotion => {
                if let Err(e) = self.cache.set_motion_data(encoded_package).await {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::SessionData => {
                if let Err(e) = self.cache.set_session_data(encoded_package).await {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::SessionHistoryData => {
                let car_id = match second_param.unwrap() {
                    OptionalMessage::Number(car_id) => car_id,
                    _ => unreachable!(),
                };

                if let Err(e) = self
                    .cache
                    .set_session_history(encoded_package, car_id)
                    .await
                {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::Participants => {
                if let Err(e) = self.cache.set_participants_data(encoded_package).await {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::EventData => {
                let string_code = match second_param.unwrap() {
                    OptionalMessage::Text(string_code) => string_code,
                    _ => unreachable!(),
                };

                if let Err(e) = self
                    .cache
                    .push_event_data(encoded_package, string_code)
                    .await
                {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::FinalClassificationData => {
                info!("Final classification data");

                self.cache.prune().await?;

                // if let Err(e) = self
                //     .cache
                //     .set_final_classification_data(&encoded_package)
                //     .await
                // {
                //     warn!("F123 cache: {}", e);
                // }
            }
        }

        Ok(())
    }

    /// Compresses data using zstd with a compression level of 3.
    ///
    /// This function takes a slice of bytes `data` as input and returns an `AppResult<Bytes>`
    /// containing the compressed data. In case of an error during compression, the error is logged,
    /// and `F123ServiceError::Compressing` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// let data = b"data to compress";
    /// let compressed_data = compress(data).unwrap();
    /// println!("Compressed data: {:?}", compressed_data);
    /// ```
    ///
    /// # Errors
    ///
    /// If compression fails, this function returns an `Err` with `F123ServiceError::Compressing`.
    ///
    /// # Parameters
    ///
    /// - `data`: A slice of bytes representing the data to be compressed.
    ///
    /// # Returns
    ///
    /// An `AppResult<Bytes>` that contains the compressed data or an error if compression fails.
    #[inline(always)]
    fn compress(data: &[u8]) -> AppResult<Bytes> {
        match zstd::stream::encode_all(data, 3) {
            Ok(compressed_data) => Ok(Bytes::from(compressed_data)),
            Err(e) => {
                warn!("Zstd compression: {}", e);
                Err(F123ServiceError::Compressing)?
            }
        }
    }
}

impl Drop for PacketBatching {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}
