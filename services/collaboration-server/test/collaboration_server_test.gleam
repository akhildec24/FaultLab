import gleeunit
import rooms

pub fn main() -> Nil {
  gleeunit.main()
}

pub fn new_room_is_empty_test() {
  let room = rooms.new_room("test-room")
  assert room.id == "test-room"
  assert room.document == ""
}

pub fn parse_join_message_test() {
  let result = rooms.parse_message("{\"type\":\"join\",\"room\":\"my-room\"}")
  assert result == Ok(rooms.JoinRoom("my-room"))
}

pub fn parse_leave_message_test() {
  let result = rooms.parse_message("{\"type\":\"leave\"}")
  assert result == Ok(rooms.LeaveRoom)
}

pub fn parse_doc_update_test() {
  let result = rooms.parse_message("{\"type\":\"doc_update\",\"data\":\"{\\\"nodes\\\":[]}\"}")
  assert result == Ok(rooms.DocUpdate("{\"nodes\":[]}"))
}

pub fn parse_cursor_test() {
  let result = rooms.parse_message("{\"type\":\"cursor\",\"data\":\"100,200\"}")
  assert result == Ok(rooms.CursorUpdate("100,200"))
}

pub fn parse_presence_test() {
  let result = rooms.parse_message("{\"type\":\"presence\",\"data\":\"Alice\"}")
  assert result == Ok(rooms.PresenceUpdate("Alice"))
}

pub fn parse_sync_response_test() {
  let result = rooms.parse_message("{\"type\":\"sync_response\",\"data\":\"doc-content\"}")
  assert result == Ok(rooms.SyncResponseMsg("doc-content"))
}

pub fn parse_invalid_message_test() {
  let result = rooms.parse_message("not json")
  assert result == Error("unexpected byte: 0x6F")
}

pub fn parse_unknown_type_test() {
  let result = rooms.parse_message("{\"type\":\"unknown\"}")
  assert result == Error("unable to decode message")
}

pub fn encode_peer_update_test() {
  let encoded = rooms.encode_message(rooms.PeerUpdate("client-1", "data"))
  assert encoded == "{\"type\":\"peer_update\",\"client_id\":\"client-1\",\"data\":\"data\"}"
}

pub fn encode_peer_joined_test() {
  let encoded = rooms.encode_message(rooms.PeerJoined("client-1"))
  assert encoded == "{\"type\":\"peer_joined\",\"client_id\":\"client-1\"}"
}

pub fn encode_sync_response_test() {
  let encoded = rooms.encode_message(rooms.SyncResponse("doc"))
  assert encoded == "{\"type\":\"sync_response\",\"data\":\"doc\"}"
}
