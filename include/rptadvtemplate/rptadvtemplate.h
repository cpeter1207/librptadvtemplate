/** @file
 * @brief Stateless, bounded UTF-8 message-template descriptor ABI.
 */
#ifndef RPTADV_TEMPLATE_H
#define RPTADV_TEMPLATE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RPTADV_TEMPLATE_ABI_VERSION 1U /**< Initial template descriptor ABI. */
#define RPTADV_TEMPLATE_OK 0 /**< Complete output or valid template. */
#define RPTADV_TEMPLATE_INVALID_SYNTAX 1 /**< Malformed or unknown placeholder. */
#define RPTADV_TEMPLATE_OUTPUT_TOO_LONG 2 /**< Output exceeds the byte bound. */
#define RPTADV_TEMPLATE_INVALID_ARGUMENT 3 /**< Invalid pointer or UTF-8 span. */

/** @brief Borrowed UTF-8 bytes; data may be null only when length is zero. */
struct rptadv_template_span {
  const uint8_t *data; /**< Borrowed, readable bytes. */
  size_t length; /**< Byte count, not Unicode character count. */
};

/** @brief Caller-owned placeholder name and worst-case replacement width. */
struct rptadv_template_key_limit {
  struct rptadv_template_span name; /**< Case-sensitive placeholder name. */
  size_t maximum; /**< Conservative replacement width in bytes. */
};

/** @brief Caller-owned placeholder name and actual replacement bytes. */
struct rptadv_template_key_value {
  struct rptadv_template_span name; /**< Case-sensitive placeholder name. */
  struct rptadv_template_span value; /**< Actual UTF-8 replacement. */
};

/** @brief Immutable stateless function table for the process lifetime. */
struct rptadv_template_descriptor {
  uint32_t struct_size; /**< Size of this descriptor. */
  uint32_t abi_version; /**< Must equal RPTADV_TEMPLATE_ABI_VERSION. */
  const char *capability_name; /**< Stable `rptadv.template` identity. */

  /** @brief Validate `${name}` syntax and conservative byte length.
   * @param text Borrowed template UTF-8 bytes.
   * @param keys Borrowed array of count records; null allowed when count is zero.
   * @param count Number of key records.
   * @param maximum Maximum complete output bytes.
   * @return RPTADV_TEMPLATE_OK or a named error code.
   */
  int32_t (*validate)(struct rptadv_template_span text,
                      const struct rptadv_template_key_limit *keys, size_t count,
                      size_t maximum);

  /** @brief Expand completely into a caller-owned output span.
   * @param text Borrowed template UTF-8 bytes.
   * @param values Borrowed array of count records; null allowed when count is zero.
   * @param count Number of value records.
   * @param output Writable capacity-byte span; null allowed when capacity is zero.
   * @param capacity Output byte bound; no terminator is written.
   * @param written Required output count on success, zero on a template error.
   * @return RPTADV_TEMPLATE_OK or a named error code; output is unchanged on error.
   * @note Input and output spans must not overlap. All nonempty spans and
   *       records remain valid for the call. No buffer is retained.
   */
  int32_t (*render)(struct rptadv_template_span text,
                    const struct rptadv_template_key_value *values, size_t count,
                    uint8_t *output, size_t capacity, size_t *written);
};

/** @brief Return the immutable ABI 1 function table; validate its prefix. */
const struct rptadv_template_descriptor *rptadv_template_descriptor(void);

#ifdef __cplusplus
}
#endif

#endif
