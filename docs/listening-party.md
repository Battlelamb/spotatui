# Listening Party

A **Listening Party** keeps the playback of a host and one or more guests in sync, so everyone hears the same track at the same position. spotatui connects them through a small WebSocket relay.

Every participant needs a Spotify login, and Spotify in spotatui needs Premium. The party follows Spotify tracks only: while the host plays another source, the room stays open but the guests get nothing new.

## Opening the party menu

Press `Ctrl+p` (the `listening_party` key) in the terminal app. The desktop app has a **Listening party** page.

## Hosting

Press `1`, `h` or `Enter` to start a party. spotatui creates a room on the relay and shows a **6-character code**. Share it with your guests.

- Your track, position and play or pause state go to the guests every 2 seconds.
- `c` switches the control mode (below).
- `l` ends the party. `Esc` only closes the menu; the party keeps running.

## Joining

Press `2` or `j`, type the 6-character code, then your display name (up to 32 characters), and press `Enter`. Your playback then follows the host. When your position drifts by more than about 3 seconds, spotatui seeks again, with the network delay taken into account. `l` leaves the party.

## Control modes

| Mode | Behavior |
| --- | --- |
| **Host only** (default) | Only the host controls playback; guests follow. |
| **Shared control** | The host accepts playback commands from guests. A guest's play/pause, next, previous and seek keys go to the host instead of this device's own playback. |

## While a party runs

The playbar shows `Party: N listeners` (host) or `Party: following <host>` (guest), and a status message tells you when someone joins or leaves.

Under Shared Control, a guest's transport keys (play/pause, next, previous, seek
forward/back) send a command to the host, which applies it and re-broadcasts
the new state to everyone — the same keys a guest would use to control their
own playback otherwise. Volume and everything outside transport (sidebar
navigation, search, and so on) always stay local to each device. If a guest's
own device is playing something the relay can't represent remotely (Apple
Music, a local file, Subsonic, YouTube, Qobuz), their transport keys control
that local playback instead, same as before joining.

## Your own relay

The default relay is `wss://spotatui-party.spotatui.workers.dev/ws`. To use your own, set it in `config.yml`:

```yaml
behavior:
  relay_server_url: "wss://relay.example.com/ws"
```
