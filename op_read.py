#automatic writing match op code to string script
import json

# Opening JSON file
with open('Opcodes.json') as json_file:
    data = json.load(json_file)["unprefixed"]
    print("match opcode {")
    for op in data:

        name = data[op]["mnemonic"]
        operands = data[op]["operands"]
        strr = f'{op} => println!("{name} '
        for operand in operands:
            strr += f' {operand["name"]} '

        strr += ' " ),'
        print(strr)
    print("}")