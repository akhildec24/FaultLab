import gleam/bytes_tree
import gleam/erlang/process
import gleam/int
import gleam/io
import gleam/option.{None, Some, type Option}
import gleam/otp/actor
import mist
import gleam/http
import gleam/http/request.{type Request}
import gleam/http/response
import rooms.{type OutMessage, type RoomMessage, type RoomState}

/// Entry point for the FaultLab collaboration server.
///
/// WebSocket server handling rooms, multiplayer editing,
/// presence information, and document synchronisation.
pub fn main() -> Nil {
  io.println("FaultLab collaboration server starting...")
  io.println("WebSocket rooms on port 4000")

  let handler = fn(req: Request(mist.Connection)) {
    case req.method {
      http.Get -> {
        case req.path {
          "/ws" -> handle_websocket(req)
          "/health" ->
            response.new(200)
            |> response.set_body(mist.Bytes(bytes_tree.from_string("{\"status\":\"ok\"}")))
          _ ->
            response.new(404)
            |> response.set_body(mist.Bytes(bytes_tree.from_string("not found")))
        }
      }
      _ ->
        response.new(405)
        |> response.set_body(mist.Bytes(bytes_tree.from_string("method not allowed")))
    }
  }

  let _ = mist.new(handler)
  |> mist.port(4000)
  |> mist.start

  process.sleep_forever()
}

/// Handle a WebSocket upgrade request.
fn handle_websocket(req: Request(mist.Connection)) {
  mist.websocket(
    req,
    on_init: fn(conn) {
      let client_id = generate_client_id()
      let inbox = process.new_subject()
      let selector =
        process.new_selector()
        |> process.select(inbox)
      let state = WebSocketState(
        client_id: client_id,
        room: None,
        conn: conn,
        inbox: inbox,
      )
      #(state, Some(selector))
    },
    on_close: fn(state) {
      case state.room {
        Some(room_subject) ->
          process.send(room_subject, rooms.Leave(state.client_id))
        None -> Nil
      }
      Nil
    },
    handler: fn(state: WebSocketState, message, _conn) {
      case message {
        mist.Text(text) -> {
          case rooms.parse_message(text) {
            Ok(rooms.JoinRoom(room_id)) -> {
              let room_name = process.new_name("room:" <> room_id)
              let builder =
                actor.new(rooms.new_room(room_id))
                |> actor.on_message(handle_actor_msg)
                |> actor.named(room_name)
              let room_actor = case actor.start(builder) {
                Ok(started) -> started.data
                Error(_) -> process.named_subject(room_name)
              }
              process.send(
                room_actor,
                rooms.Join(state.client_id, state.inbox),
              )
              mist.continue(WebSocketState(..state, room: Some(room_actor)))
            }

            Ok(rooms.DocUpdate(data)) -> {
              case state.room {
                Some(room) ->
                  process.send(room, rooms.Update(state.client_id, data))
                None -> Nil
              }
              mist.continue(state)
            }

            Ok(rooms.CursorUpdate(data)) -> {
              case state.room {
                Some(room) ->
                  process.send(room, rooms.Cursor(state.client_id, data))
                None -> Nil
              }
              mist.continue(state)
            }

            Ok(rooms.PresenceUpdate(data)) -> {
              case state.room {
                Some(room) ->
                  process.send(room, rooms.Presence(state.client_id, data))
                None -> Nil
              }
              mist.continue(state)
            }

            Ok(rooms.SyncResponseMsg(data)) -> {
              case state.room {
                Some(room) ->
                  process.send(room, rooms.Update(state.client_id, data))
                None -> Nil
              }
              mist.continue(state)
            }

            Ok(rooms.LeaveRoom) -> {
              case state.room {
                Some(room) ->
                  process.send(room, rooms.Leave(state.client_id))
                None -> Nil
              }
              mist.continue(WebSocketState(..state, room: None))
            }

            Error(_) -> {
              let _ = mist.send_text_frame(
                state.conn,
                "{\"type\":\"error\",\"message\":\"invalid message\"}",
              )
              mist.continue(state)
            }
          }
        }

        mist.Binary(_) -> mist.continue(state)

        mist.Closed -> mist.stop()

        mist.Shutdown -> mist.stop()

        mist.Custom(out_msg) -> {
          let _ = mist.send_text_frame(state.conn, rooms.encode_message(out_msg))
          mist.continue(state)
        }
      }
    },
  )
}

/// WebSocket connection state per client.
type WebSocketState {
  WebSocketState(
    client_id: String,
    room: Option(process.Subject(RoomMessage)),
    conn: mist.WebsocketConnection,
    inbox: process.Subject(OutMessage),
  )
}

/// Actor message handler for room actors.
fn handle_actor_msg(state: RoomState, msg: RoomMessage) -> actor.Next(RoomState, RoomMessage) {
  let new_state = rooms.handle_room_message(msg, state)
  case msg {
    rooms.Join(_client_id, subject) -> {
      case state.document != "" {
        True -> process.send(subject, rooms.SyncResponse(state.document))
        False -> Nil
      }
      actor.continue(new_state)
    }
    _ -> actor.continue(new_state)
  }
}

/// Generate a unique client ID.
fn generate_client_id() -> String {
  "client-" <> int.to_string(client_counter)
}

/// Simple counter for client IDs.
const client_counter = 0
