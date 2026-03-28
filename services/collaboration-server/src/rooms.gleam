import gleam/dict.{type Dict}
import gleam/erlang/process.{type Subject}
import gleam/json
import gleam/dynamic/decode
import gleam/result

/// A connected client identified by their WebSocket subject.
pub type ClientId =
  String

/// A room identified by its name.
pub type RoomId =
  String

/// Messages that the room actor can receive.
pub type RoomMessage {
  /// A new client joins the room.
  Join(ClientId, Subject(OutMessage))
  /// A client leaves the room.
  Leave(ClientId)
  /// A client sends a document update to broadcast.
  Update(ClientId, String)
  /// A client sends a cursor position update.
  Cursor(ClientId, String)
  /// A client sends a presence update.
  Presence(ClientId, String)
}

/// Messages sent out to connected clients.
pub type OutMessage {
  /// Broadcast a document update from a peer.
  PeerUpdate(ClientId, String)
  /// Broadcast a cursor position from a peer.
  PeerCursor(ClientId, String)
  /// Broadcast a presence update from a peer.
  PeerPresence(ClientId, String)
  /// Notify that a peer joined.
  PeerJoined(ClientId)
  /// Notify that a peer left.
  PeerLeft(ClientId)
  /// Request full document sync (sent to existing clients when a new peer joins).
  SyncRequest(ClientId)
  /// Full document sync response.
  SyncResponse(String)
}

/// State held by a room actor.
pub type RoomState {
  RoomState(
    id: RoomId,
    clients: Dict(ClientId, Subject(OutMessage)),
    document: String,
  )
}

/// Create a new empty room.
pub fn new_room(id: RoomId) -> RoomState {
  RoomState(id: id, clients: dict.new(), document: "")
}

/// Handle a room message — pure function for testability.
pub fn handle_room_message(state: RoomMessage, room: RoomState) -> RoomState {
  case state {
    Join(client_id, subject) -> {
      let clients = dict.insert(room.clients, client_id, subject)
      // Notify all existing clients that a new peer joined
      broadcast(clients, client_id, PeerJoined(client_id))
      // Ask existing clients to send sync data to the new peer
      broadcast(clients, client_id, SyncRequest(client_id))
      RoomState(..room, clients: clients)
    }

    Leave(client_id) -> {
      let clients = dict.delete(room.clients, client_id)
      broadcast(clients, client_id, PeerLeft(client_id))
      RoomState(..room, clients: clients)
    }

    Update(client_id, data) -> {
      // Broadcast to all other clients
      broadcast(room.clients, client_id, PeerUpdate(client_id, data))
      RoomState(..room, document: data)
    }

    Cursor(client_id, data) -> {
      broadcast(room.clients, client_id, PeerCursor(client_id, data))
      room
    }

    Presence(client_id, data) -> {
      broadcast(room.clients, client_id, PeerPresence(client_id, data))
      room
    }
  }
}

/// Send a message to all clients except the sender.
fn broadcast(
  clients: Dict(ClientId, Subject(OutMessage)),
  sender: ClientId,
  msg: OutMessage,
) -> Nil {
  clients
  |> dict.each(fn(id, subject) {
    case id != sender {
      True -> process.send(subject, msg)
      False -> Nil
    }
  })
}

/// Encode an OutMessage as JSON for sending over WebSocket.
pub fn encode_message(msg: OutMessage) -> String {
  case msg {
    PeerUpdate(client_id, data) ->
      json.object([
        #("type", json.string("peer_update")),
        #("client_id", json.string(client_id)),
        #("data", json.string(data)),
      ])
      |> json.to_string

    PeerCursor(client_id, data) ->
      json.object([
        #("type", json.string("peer_cursor")),
        #("client_id", json.string(client_id)),
        #("data", json.string(data)),
      ])
      |> json.to_string

    PeerPresence(client_id, data) ->
      json.object([
        #("type", json.string("peer_presence")),
        #("client_id", json.string(client_id)),
        #("data", json.string(data)),
      ])
      |> json.to_string

    PeerJoined(client_id) ->
      json.object([
        #("type", json.string("peer_joined")),
        #("client_id", json.string(client_id)),
      ])
      |> json.to_string

    PeerLeft(client_id) ->
      json.object([
        #("type", json.string("peer_left")),
        #("client_id", json.string(client_id)),
      ])
      |> json.to_string

    SyncRequest(client_id) ->
      json.object([
        #("type", json.string("sync_request")),
        #("client_id", json.string(client_id)),
      ])
      |> json.to_string

    SyncResponse(data) ->
      json.object([
        #("type", json.string("sync_response")),
        #("data", json.string(data)),
      ])
      |> json.to_string
  }
}

/// A parsed incoming message from a client.
pub type ParsedMessage {
  JoinRoom(RoomId)
  LeaveRoom
  DocUpdate(String)
  CursorUpdate(String)
  PresenceUpdate(String)
  SyncResponseMsg(String)
}

/// Parse an incoming JSON message from a client.
pub fn parse_message(raw: String) -> Result(ParsedMessage, String) {
  json.parse(raw, message_decoder())
  |> result.map_error(fn(err) {
    case err {
      json.UnexpectedEndOfInput -> "unexpected end of input"
      json.UnexpectedByte(s) -> "unexpected byte: " <> s
      json.UnexpectedSequence(s) -> "unexpected sequence: " <> s
      json.UnableToDecode(_) -> "unable to decode message"
    }
  })
}

/// JSON decoder for incoming client messages.
fn message_decoder() -> decode.Decoder(ParsedMessage) {
  use msg_type <- decode.field("type", decode.string)
  case msg_type {
    "join" -> {
      use room_id <- decode.field("room", decode.string)
      decode.success(JoinRoom(room_id))
    }
    "leave" -> decode.success(LeaveRoom)
    "doc_update" -> {
      use doc_data <- decode.field("data", decode.string)
      decode.success(DocUpdate(doc_data))
    }
    "cursor" -> {
      use cursor_data <- decode.field("data", decode.string)
      decode.success(CursorUpdate(cursor_data))
    }
    "presence" -> {
      use presence_data <- decode.field("data", decode.string)
      decode.success(PresenceUpdate(presence_data))
    }
    "sync_response" -> {
      use sync_data <- decode.field("data", decode.string)
      decode.success(SyncResponseMsg(sync_data))
    }
    _ -> decode.failure(JoinRoom(""), "valid message type")
  }
}
