// Jackson Coxson

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Callback signature for custom logging.
 * Return true if the log was consumed by the caller, false to fall back to internal logging.
 * level: 0 = DEBUG, 1 = INFO, 2 = WARN, 3 = ERROR
 */
typedef bool (*EMProxyLogCallback)(int level, const char *message);

/**
 * Sets a custom logging callback function.
 */
void set_log_callback(EMProxyLogCallback cb);

/**
 * Starts your emotional damage
 * # Arguments
 * * `bind_addr` - The UDP socket to listen to
 * # Returns
 * 0 on success, -1 if null address, -2 if UTF-8 error, -3 if invalid socket address, -4 if socket bind failed, -5 if crypto init failed
 * # Safety
 * Don't be stupid
 */
int start_emotional_damage(const char *bind_addr);

/**
 * Stops further emotional damage
 * # Returns
 * 0 on success, -1 if no server running, -2 if failed to send stop signal
 * # Safety
 * Don't be stupid
 */
int stop_emotional_damage(void);

/**
 * Blocks until Wireguard is ready
 * # Arguments
 * * `timeout` - The timeout in miliseconds to wait for Wireguard
 * # Returns
 * 0 on success, -1 on failure
 */
int test_emotional_damage(int timeout);
