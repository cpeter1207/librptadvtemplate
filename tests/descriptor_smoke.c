#include "rptadvtemplate/rptadvtemplate.h"

#include <assert.h>
#include <string.h>

static struct rptadv_template_span span(const char *text) {
  struct rptadv_template_span value = {(const uint8_t *)text, strlen(text)};
  return value;
}

int main(void) {
  const struct rptadv_template_descriptor *api = rptadv_template_descriptor();
  assert(api == rptadv_template_descriptor());
  assert(api->struct_size == sizeof(*api));
  assert(api->abi_version == RPTADV_TEMPLATE_ABI_VERSION);
  assert(strcmp(api->capability_name, "rptadv.template") == 0);
  assert(api->validate && api->render);

  const struct rptadv_template_key_limit keys[] = {{
      span("name"), 4}};
  assert(api->validate(span("Hi ${name}!"), keys, 1, 8) == RPTADV_TEMPLATE_OK);
  assert(api->validate(span("${unknown}"), keys, 1, 127) ==
         RPTADV_TEMPLATE_INVALID_SYNTAX);
  const struct rptadv_template_key_value values[] = {{
      span("name"), span("J\xc3\xb6hn")}};
  uint8_t output[9] = {0};
  size_t written = 123;
  assert(api->render(span("Hi ${name}!"), values, 1, output,
                     sizeof(output), &written) == RPTADV_TEMPLATE_OK);
  assert(written == 9);
  assert(memcmp(output, "Hi J\xc3\xb6hn!", written) == 0);
  written = 123;
  assert(api->render(span("Hi ${name}!"), values, 1, output, 8,
                     &written) == RPTADV_TEMPLATE_OUTPUT_TOO_LONG);
  assert(written == 0);
  return 0;
}
