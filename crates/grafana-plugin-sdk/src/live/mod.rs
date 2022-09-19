/*! Grafana Live features, providing streaming functionality to Grafana.

The types in this module are used by Grafana Live, a real-time messaging engine introduced
in Grafana v8.0. Grafana Live allows you to push event data to a frontend as soon as the event occurs.
Generally this will be done by plugins that implement the [`StreamService`][crate::backend::StreamService]
trait.

See the [Grafana Live documentation](https://grafana.com/docs/grafana/latest/live/) for more information.

*/
mod channel;

pub use channel::{Channel, Error as ChannelError, Namespace, Path, Scope, MAX_CHANNEL_LENGTH};
