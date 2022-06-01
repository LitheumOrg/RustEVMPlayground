// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`.

// ignore_for_file: non_constant_identifier_names, unused_element, duplicate_ignore, directives_ordering, curly_braces_in_flow_control_structures, unnecessary_lambdas, slash_for_doc_comments, prefer_const_literals_to_create_immutables, implicit_dynamic_list_literal, duplicate_import, unused_import, prefer_single_quotes, prefer_const_constructors

import 'dart:convert';
import 'dart:typed_data';

import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'dart:ffi' as ffi;

abstract class Native {
  Future<String> greet({dynamic hint});

  Future<Uint8List> generateKeypair({dynamic hint});

  Future<String> getAddress({required Uint8List slice, dynamic hint});

  Future<int> getBalance({dynamic hint});
}

class NativeImpl extends FlutterRustBridgeBase<NativeWire> implements Native {
  factory NativeImpl(ffi.DynamicLibrary dylib) =>
      NativeImpl.raw(NativeWire(dylib));

  NativeImpl.raw(NativeWire inner) : super(inner);

  Future<String> greet({dynamic hint}) => executeNormal(FlutterRustBridgeTask(
        callFfi: (port_) => inner.wire_greet(port_),
        parseSuccessData: _wire2api_String,
        constMeta: const FlutterRustBridgeTaskConstMeta(
          debugName: "greet",
          argNames: [],
        ),
        argValues: [],
        hint: hint,
      ));

  Future<Uint8List> generateKeypair({dynamic hint}) =>
      executeNormal(FlutterRustBridgeTask(
        callFfi: (port_) => inner.wire_generate_keypair(port_),
        parseSuccessData: _wire2api_uint_8_list,
        constMeta: const FlutterRustBridgeTaskConstMeta(
          debugName: "generate_keypair",
          argNames: [],
        ),
        argValues: [],
        hint: hint,
      ));

  Future<String> getAddress({required Uint8List slice, dynamic hint}) =>
      executeNormal(FlutterRustBridgeTask(
        callFfi: (port_) =>
            inner.wire_get_address(port_, _api2wire_uint_8_list(slice)),
        parseSuccessData: _wire2api_String,
        constMeta: const FlutterRustBridgeTaskConstMeta(
          debugName: "get_address",
          argNames: ["slice"],
        ),
        argValues: [slice],
        hint: hint,
      ));

  Future<int> getBalance({dynamic hint}) => executeNormal(FlutterRustBridgeTask(
        callFfi: (port_) => inner.wire_get_balance(port_),
        parseSuccessData: _wire2api_u64,
        constMeta: const FlutterRustBridgeTaskConstMeta(
          debugName: "get_balance",
          argNames: [],
        ),
        argValues: [],
        hint: hint,
      ));

  // Section: api2wire
  int _api2wire_u8(int raw) {
    return raw;
  }

  ffi.Pointer<wire_uint_8_list> _api2wire_uint_8_list(Uint8List raw) {
    final ans = inner.new_uint_8_list(raw.length);
    ans.ref.ptr.asTypedList(raw.length).setAll(0, raw);
    return ans;
  }

  // Section: api_fill_to_wire

}

// Section: wire2api
String _wire2api_String(dynamic raw) {
  return raw as String;
}

int _wire2api_u64(dynamic raw) {
  return raw as int;
}

int _wire2api_u8(dynamic raw) {
  return raw as int;
}

Uint8List _wire2api_uint_8_list(dynamic raw) {
  return raw as Uint8List;
}

// ignore_for_file: camel_case_types, non_constant_identifier_names, avoid_positional_boolean_parameters, annotate_overrides, constant_identifier_names

// AUTO GENERATED FILE, DO NOT EDIT.
//
// Generated by `package:ffigen`.

/// generated by flutter_rust_bridge
class NativeWire implements FlutterRustBridgeWireBase {
  /// Holds the symbol lookup function.
  final ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
      _lookup;

  /// The symbols are looked up in [dynamicLibrary].
  NativeWire(ffi.DynamicLibrary dynamicLibrary)
      : _lookup = dynamicLibrary.lookup;

  /// The symbols are looked up with [lookup].
  NativeWire.fromLookup(
      ffi.Pointer<T> Function<T extends ffi.NativeType>(String symbolName)
          lookup)
      : _lookup = lookup;

  void wire_greet(
    int port_,
  ) {
    return _wire_greet(
      port_,
    );
  }

  late final _wire_greetPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>('wire_greet');
  late final _wire_greet = _wire_greetPtr.asFunction<void Function(int)>();

  void wire_generate_keypair(
    int port_,
  ) {
    return _wire_generate_keypair(
      port_,
    );
  }

  late final _wire_generate_keypairPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>(
          'wire_generate_keypair');
  late final _wire_generate_keypair =
      _wire_generate_keypairPtr.asFunction<void Function(int)>();

  void wire_get_address(
    int port_,
    ffi.Pointer<wire_uint_8_list> slice,
  ) {
    return _wire_get_address(
      port_,
      slice,
    );
  }

  late final _wire_get_addressPtr = _lookup<
      ffi.NativeFunction<
          ffi.Void Function(
              ffi.Int64, ffi.Pointer<wire_uint_8_list>)>>('wire_get_address');
  late final _wire_get_address = _wire_get_addressPtr
      .asFunction<void Function(int, ffi.Pointer<wire_uint_8_list>)>();

  void wire_get_balance(
    int port_,
  ) {
    return _wire_get_balance(
      port_,
    );
  }

  late final _wire_get_balancePtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(ffi.Int64)>>(
          'wire_get_balance');
  late final _wire_get_balance =
      _wire_get_balancePtr.asFunction<void Function(int)>();

  ffi.Pointer<wire_uint_8_list> new_uint_8_list(
    int len,
  ) {
    return _new_uint_8_list(
      len,
    );
  }

  late final _new_uint_8_listPtr = _lookup<
      ffi.NativeFunction<
          ffi.Pointer<wire_uint_8_list> Function(
              ffi.Int32)>>('new_uint_8_list');
  late final _new_uint_8_list = _new_uint_8_listPtr
      .asFunction<ffi.Pointer<wire_uint_8_list> Function(int)>();

  void free_WireSyncReturnStruct(
    WireSyncReturnStruct val,
  ) {
    return _free_WireSyncReturnStruct(
      val,
    );
  }

  late final _free_WireSyncReturnStructPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(WireSyncReturnStruct)>>(
          'free_WireSyncReturnStruct');
  late final _free_WireSyncReturnStruct = _free_WireSyncReturnStructPtr
      .asFunction<void Function(WireSyncReturnStruct)>();

  void store_dart_post_cobject(
    DartPostCObjectFnType ptr,
  ) {
    return _store_dart_post_cobject(
      ptr,
    );
  }

  late final _store_dart_post_cobjectPtr =
      _lookup<ffi.NativeFunction<ffi.Void Function(DartPostCObjectFnType)>>(
          'store_dart_post_cobject');
  late final _store_dart_post_cobject = _store_dart_post_cobjectPtr
      .asFunction<void Function(DartPostCObjectFnType)>();
}

class wire_uint_8_list extends ffi.Struct {
  external ffi.Pointer<ffi.Uint8> ptr;

  @ffi.Int32()
  external int len;
}

typedef DartPostCObjectFnType = ffi.Pointer<
    ffi.NativeFunction<ffi.Uint8 Function(DartPort, ffi.Pointer<ffi.Void>)>>;
typedef DartPort = ffi.Int64;
