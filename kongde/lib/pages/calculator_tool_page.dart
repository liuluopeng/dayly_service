import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:kongde/src/rust/api/utils/calculator.dart';
import 'package:kongde/widgets/common_app_bar.dart';

class CalculatorToolPage extends StatefulWidget {
  const CalculatorToolPage({super.key});

  @override
  State<CalculatorToolPage> createState() => _CalculatorToolPageState();
}

class _CalculatorToolPageState extends State<CalculatorToolPage> {
  int calculatorA = 0;
  int calculatorB = 0;
  int addResult = 0;
  int add22Result = 0;
  int multiplyResult = 0;

  void _calculate() {
    setState(() {
      addResult = add(a: calculatorA, b: calculatorB);
      add22Result = add22(a: calculatorA, b: calculatorB);
      multiplyResult = multiply(a: calculatorA, b: calculatorB);
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: CommonAppBar(title: 'calculator.title'.tr),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                Expanded(
                  child: TextField(
                    onChanged: (value) {
                      setState(() {
                        calculatorA = int.tryParse(value) ?? 0;
                      });
                    },
                    keyboardType: TextInputType.number,
                    decoration: InputDecoration(
                      border: const OutlineInputBorder(),
                      labelText: 'calculator.numberA'.tr,
                    ),
                  ),
                ),
                const SizedBox(width: 16.0),
                Expanded(
                  child: TextField(
                    onChanged: (value) {
                      setState(() {
                        calculatorB = int.tryParse(value) ?? 0;
                      });
                    },
                    keyboardType: TextInputType.number,
                    decoration: InputDecoration(
                      border: const OutlineInputBorder(),
                      labelText: 'calculator.numberB'.tr,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16.0),
            ElevatedButton(onPressed: _calculate, child: Text('calculator.calculate'.tr)),
            const SizedBox(height: 16.0),
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'calculator.result'.tr,
                      style: const TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 8.0),
                    Text('A + B + 10 = $addResult'),
                    Text('A + B + 22 = $add22Result'),
                    Text('A × B = $multiplyResult'),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
      ),
    );
  }
}
