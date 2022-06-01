import 'dart:typed_data';
import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:fast_base58/fast_base58.dart';
import 'ffi.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Litheum Wallet',
      theme: ThemeData(
        // This is the theme of your application.
        //
        // Try running your application with "flutter run". You'll see the
        // application has a blue toolbar. Then, without quitting the app, try
        // changing the primarySwatch below to Colors.green and then invoke
        // "hot reload" (press "r" in the console where you ran "flutter run",
        // or simply save your changes to "hot reload" in a Flutter IDE).
        // Notice that the counter didn't reset back to zero; the application
        // is not restarted.
        // primarySwatch: Colors.blue,
        colorScheme: ColorScheme.fromSwatch().copyWith(
          primary: const Color(0xffeaff55),
          // secondary: const Color(0xFFFFC107),
        ),
      ),
      home: const MyHomePage(title: 'Litheum Wallet'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({Key? key, required this.title}) : super(key: key);

  // This widget is the home page of your application. It is stateful, meaning
  // that it has a State object (defined below) that contains fields that affect
  // how it looks.

  // This class is the configuration for the state. It holds the values (in this
  // case the title) provided by the parent (in this case the App widget) and
  // used by the build method of the State. Fields in a Widget subclass are
  // always marked "final".

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  // These futures belong to the state and are only initialized once,
  // in the initState method.
  // late Future<Platform> platform;
  // late Future<bool> isRelease;
  final _storage = const FlutterSecureStorage();
  late Future<String> greeter;
  late Future<String> address;
  late Future<int> balance;

  @override
  void initState() {
    super.initState();

    // _storage.deleteAll();

    _readAll();
    _initKeypairStore();
    greeter = api.greet();
    address = Future<String>.delayed(const Duration(seconds: 1), () => '0x');
    balance = api.getBalance();
  }

  Future<void> _readAll() async {
    final all = await _storage.readAll(
        // iOptions: _getIOSOptions(),
        aOptions: _getAndroidOptions());
    setState(() {
      all.entries.map((entry) {
        print('${entry.key}: ${entry.value}');
      });
    });
  }

  Future<void> _initKeypairStore() async {
    var keypair = await _storage.read(key: 'keypair_store');
    if (keypair == null) {
      Uint8List _keypair = await api.generateKeypair();
      final test = Base58Encode(_keypair);
      print('encrypted_key from Rust: $_keypair');
      print('Base58Encode keypair_store: $test');
      final test1 = Base58Decode(test);
      print('Base58Decode keypair_store: $test1');
      await _storage.write(
          // key: 'keypair', value: keypair.buffer.asByteData() as String);
          key: 'keypair_store',
          value: Base58Encode(_keypair));
      address = api.getAddress(slice: _keypair);
    } else {
      print('keypair_store 1: $keypair');
      final test = Base58Decode(keypair);
      print('Base58Decode keypair_store: $test');
      final slice = Uint8List.fromList(Base58Decode(keypair));
      print('origianl slice: $slice');
      address =
          api.getAddress(slice: Uint8List.fromList(Base58Decode(keypair)));
    }
  }

  // IOSOptions _getIOSOptions() => IOSOptions();

  AndroidOptions _getAndroidOptions() => const AndroidOptions(
        encryptedSharedPreferences: true,
      );

  @override
  Widget build(BuildContext context) {
    // This method is rerun every time setState is called, for instance as done
    // by the _incrementCounter method above.
    //
    // The Flutter framework has been optimized to make rerunning build methods
    // fast, so that you can just rebuild anything that needs updating rather
    // than having to individually change instances of widgets.
    return Scaffold(
      appBar: AppBar(
        // Here we take the value from the MyHomePage object that was created by
        // the App.build method, and use it to set our appbar title.
        title: Text(widget.title),
      ),
      body: Center(
        // Center is a layout widget. It takes a single child and positions it
        // in the middle of the parent.
        child: Column(
          // Column is also a layout widget. It takes a list of children and
          // arranges them vertically. By default, it sizes itself to fit its
          // children horizontally, and tries to be as tall as its parent.
          //
          // Invoke "debug painting" (press "p" in the console, choose the
          // "Toggle Debug Paint" action from the Flutter Inspector in Android
          // Studio, or the "Toggle Debug Paint" command in Visual Studio Code)
          // to see the wireframe for each widget.
          //
          // Column has various properties to control how it sizes itself and
          // how it positions its children. Here we use mainAxisAlignment to
          // center the children vertically; the main axis here is the vertical
          // axis because Columns are vertical (the cross axis would be
          // horizontal).
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            // const Text("Keypair Address:"),
            const Text("Keypair Address | Balance:"),
            // To render the results of a Future, a FutureBuilder is used which
            // turns a Future into an AsyncSnapshot, which can be used to
            // extract the error state, the loading state and the data if
            // available.
            //
            // Here, the generic type that the FutureBuilder manages is
            // explicitly named, because if omitted the snapshot will have the
            // type of AsyncSnapshot<Object?>.
            FutureBuilder<List<dynamic>>(
              // We await two unrelated futures here, so the type has to be
              // List<dynamic>.
              future: Future.wait([greeter, address, balance]),
              builder: (context, snap) {
                final style = Theme.of(context).textTheme.headline4;
                if (snap.error != null) {
                  // An error has been encountered, so give an appropriate response and
                  // pass the error details to an unobstructive tooltip.
                  debugPrint(snap.error.toString());
                  return Tooltip(
                    message: snap.error.toString(),
                    child: Text('Unknown OS', style: style),
                  );
                }

                // Guard return here, the data is not ready yet.
                final data = snap.data;
                if (data == null) return const CircularProgressIndicator();

                // Finally, retrieve the data expected in the same order provided
                // to the FutureBuilder.future.
                final text0 = data[1];
                final text1 = data[2];
                return Text('$text0 | $text1', style: style);
              },
            )
          ],
        ),
      ),
    );
  }
}
