const COMMANDS: &[&str] = &[
   "get",
   "set",
   "remove",
   "keys",
   "get_all",
   "synchronize",
   "account_status",
];

fn main() {
   tauri_plugin::Builder::new(COMMANDS).build();
}
