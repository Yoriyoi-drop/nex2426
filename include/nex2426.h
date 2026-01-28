#ifndef NEX2426_H
#define NEX2426_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stddef.h>

/* Opaque handle for NEX2426 kernel instance */
typedef struct NexKernel NexKernel;

/* Result structure for C API operations */
typedef struct {
    /* Success status (0 = success, non-zero = error) */
    int status;
    /* Result hash string (null-terminated) */
    char* hash;
    /* Error message (null-terminated, null if no error) */
    char* error;
    /* Timestamp from the operation */
    uint64_t timestamp;
} NexResult;

/* Configuration structure for kernel creation */
typedef struct {
    /* Cost parameter (1-10) */
    uint32_t cost;
    /* Enable temporal binding (0 = false, 1 = true) */
    int temporal_binding;
    /* Reserved for future use */
    uint32_t reserved[8];
} NexConfig;

/* Default configuration values */
#define NEX_DEFAULT_COST 3
#define NEX_DEFAULT_TEMPORAL_BINDING 0
#define NEX_MAX_COST 10

/* Error codes */
#define NEX_SUCCESS 0
#define NEX_ERROR_INVALID_PARAMS -1
#define NEX_ERROR_INVALID_ENCODING -2
#define NEX_ERROR_MEMORY_ALLOCATION -3
#define NEX_ERROR_INTERNAL -4

/* Library management functions */

/**
 * Create a new NEX2426 kernel instance
 * 
 * @param config Configuration for the kernel
 * @return Pointer to kernel instance, or NULL on error
 */
NexKernel* nex_kernel_create(const NexConfig* config);

/**
 * Destroy a NEX2426 kernel instance
 * 
 * @param kernel Pointer to kernel instance
 */
void nex_kernel_destroy(NexKernel* kernel);

/* Hashing functions */

/**
 * Hash data using NEX2426 kernel
 * 
 * @param kernel Pointer to kernel instance
 * @param data Pointer to input data
 * @param data_len Length of input data in bytes
 * @param key Pointer to encryption key (null-terminated string)
 * @return Result structure containing hash or error information
 */
NexResult nex_hash_data(
    NexKernel* kernel,
    const uint8_t* data,
    size_t data_len,
    const char* key
);

/**
 * Hash string using NEX2426 kernel
 * 
 * @param kernel Pointer to kernel instance
 * @param data Pointer to input string (null-terminated)
 * @param key Pointer to encryption key (null-terminated string)
 * @return Result structure containing hash or error information
 */
NexResult nex_hash_string(
    NexKernel* kernel,
    const char* data,
    const char* key
);

/* Memory management functions */

/**
 * Free result structure memory
 * 
 * @param result Pointer to result structure
 */
void nex_result_free(NexResult* result);

/**
 * Free string allocated by library
 * 
 * @param string Pointer to string (null-terminated)
 */
void nex_free_string(char* string);

/* Library information functions */

/**
 * Get library version information
 * 
 * @return Pointer to version string (null-terminated)
 */
const char* nex_get_version(void);

/**
 * Get library build information
 * 
 * @return Pointer to build info string (null-terminated)
 */
const char* nex_get_build_info(void);

/* Performance functions */

/**
 * Performance benchmark function
 * 
 * @param kernel Pointer to kernel instance
 * @param iterations Number of iterations to run
 * @param data_size Size of test data in bytes
 * @return Average time per iteration in nanoseconds
 */
uint64_t nex_benchmark(
    NexKernel* kernel,
    uint32_t iterations,
    size_t data_size
);

/* Utility macros for common operations */

#define NEX_CONFIG_INIT { \
    .cost = NEX_DEFAULT_COST, \
    .temporal_binding = NEX_DEFAULT_TEMPORAL_BINDING, \
    .reserved = {0} \
}

#define NEX_RESULT_INIT { \
    .status = NEX_ERROR_INTERNAL, \
    .hash = NULL, \
    .error = NULL, \
    .timestamp = 0 \
}

/* Convenience functions for common use cases */

/**
 * Simple hash function with default configuration
 * 
 * @param data Input string to hash
 * @param key Encryption key
 * @return Hash string (caller must free with nex_free_string), or NULL on error
 */
static inline char* nex_hash_simple(const char* data, const char* key) {
    NexConfig config = NEX_CONFIG_INIT;
    NexKernel* kernel = nex_kernel_create(&config);
    if (!kernel) return NULL;
    
    NexResult result = nex_hash_string(kernel, data, key);
    char* hash = NULL;
    
    if (result.status == NEX_SUCCESS && result.hash) {
        hash = result.hash;
        result.hash = NULL; /* Prevent double free */
    }
    
    nex_result_free(&result);
    nex_kernel_destroy(kernel);
    return hash;
}

/**
 * Hash data with custom cost parameter
 * 
 * @param data Input string to hash
 * @param key Encryption key
 * @param cost Cost parameter (1-10)
 * @return Hash string (caller must free with nex_free_string), or NULL on error
 */
static inline char* nex_hash_with_cost(const char* data, const char* key, uint32_t cost) {
    NexConfig config = NEX_CONFIG_INIT;
    config.cost = cost;
    
    NexKernel* kernel = nex_kernel_create(&config);
    if (!kernel) return NULL;
    
    NexResult result = nex_hash_string(kernel, data, key);
    char* hash = NULL;
    
    if (result.status == NEX_SUCCESS && result.hash) {
        hash = result.hash;
        result.hash = NULL; /* Prevent double free */
    }
    
    nex_result_free(&result);
    nex_kernel_destroy(kernel);
    return hash;
}

/**
 * Hash data with temporal binding enabled
 * 
 * @param data Input string to hash
 * @param key Encryption key
 * @return Hash string (caller must free with nex_free_string), or NULL on error
 */
static inline char* nex_hash_temporal(const char* data, const char* key) {
    NexConfig config = NEX_CONFIG_INIT;
    config.temporal_binding = 1;
    
    NexKernel* kernel = nex_kernel_create(&config);
    if (!kernel) return NULL;
    
    NexResult result = nex_hash_string(kernel, data, key);
    char* hash = NULL;
    
    if (result.status == NEX_SUCCESS && result.hash) {
        hash = result.hash;
        result.hash = NULL; /* Prevent double free */
    }
    
    nex_result_free(&result);
    nex_kernel_destroy(kernel);
    return hash;
}

/* Error handling utilities */

/**
 * Check if result indicates success
 * 
 * @param result Result structure
 * @return 1 if successful, 0 otherwise
 */
static inline int nex_result_is_success(const NexResult* result) {
    return result && result->status == NEX_SUCCESS;
}

/**
 * Get error message from result
 * 
 * @param result Result structure
 * @return Error message string, or NULL if no error
 */
static inline const char* nex_result_get_error(const NexResult* result) {
    return (result && result->error) ? result->error : "No error";
}

/**
 * Get hash from result
 * 
 * @param result Result structure
 * @return Hash string, or NULL if no hash
 */
static inline const char* nex_result_get_hash(const NexResult* result) {
    return (result && result->hash) ? result->hash : NULL;
}

#ifdef __cplusplus
}
#endif

#endif /* NEX2426_H */
