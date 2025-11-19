// Cross-platform export macro for TCP Lab C++ protocol libraries.
// Use this macro on functions that need to be visible from the shared library
// (e.g. create_sender, sender_init, sender_on_packet, ...).
//
// Example:
//   TCP_LAB_EXPORT ouc::tcp::sdk::TransportProtocol* create_sender();
//

#pragma once

#if defined(_WIN32) || defined(_WIN64)
  #define TCP_LAB_EXPORT extern "C" __declspec(dllexport)
#else
  #define TCP_LAB_EXPORT extern "C"
#endif


