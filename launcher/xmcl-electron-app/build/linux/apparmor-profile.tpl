abi <abi/4.0>,
include <tunables/global>

profile "${executable}" "/opt/sawbyte-launcher/${executable}" flags=(unconfined) {
  userns,

  include if exists <local/${executable}>
}
