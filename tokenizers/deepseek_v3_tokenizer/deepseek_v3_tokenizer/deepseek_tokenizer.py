# Creado por: DeepSeek
# Descripción: El siguiente código ha sido modificado para contar el número de tokens generados a partir del texto de entrada más el contrato en el prompt,
# para los contratos analizados por el modelo deepseek V3.1
import transformers
from pathlib import Path

chat_tokenizer_dir = "./"

tokenizer = transformers.AutoTokenizer.from_pretrained( 
    chat_tokenizer_dir, trust_remote_code=True
)

folder_wallet = Path("../wallet-program/programs/vulnerable-programs-dataset/programs-AI")
folder_voting = Path("../voting-program/programs/vulnerable-programs-dataset/programs-AI")

def prompt_complex(program_code):

    complex_prompt =  f"""You are an expert developer of smart contracts on Solana using Rust,
        with deep knowledge of the most advanced techniques available today. 
        Your task is to analyze the Rust code provided in the prompt for potential vulnerabilities.
        Identify functions that are deprecated, insecure, or pose potential risks, 
        and provide clear explanations for each finding.
        Your output must strictly follow this format. 
        For every function in the code, list its name and describe its characteristics 
        or vulnerabilities using bullet points:
        Function \"function_name\":
        - Bullet point 1
        - Bullet point 2
        - ...
        Function \"function_name\":
        - Bullet point 1
        - Bullet point 2
        - ...
        Function \"function_name\":
        - Bullet point 1
        - Bullet point 2
        - ...
        ...
        Rules:
        - Analyze each function you find in the code.
        - Use the exact function name inside quotes after the word Function.
        - Write only bullet points under each function.
        - Do not include explanations, summaries, or text outside this format.
        
        The following program is about to be deployed on Solana´s blockchain. 
        Please review it for any potential security vulnerabilities and 
        suggest ways to mitigate them. Program:\n\n{program_code}"""

    return complex_prompt

def prompt_basic(program_code):
    basic_prompt = system_message = f"""You are a helpful assistant.
        Your output must strictly follow this format. 
        For every function you find in the code, 
        include its name and describe it using bullet points:
        Function \"function_name\":
        - Bullet point 1
        - Bullet point 2
        - ...
        Function \"function_name\":
        - Bullet point 1
        - Bullet point 2
        - ...
        Function \"function_name\":
        - Bullet point 1
        - Bullet point 2
        - ...
        Rules:
        - Analyse each function you find in the code.
        - Write your findings only as bullet points under each function.
        - Do not add text, explanations, or summaries outside this format.
        
        Is this program correct? Search for potential risks and vulnerabilities. Program:\n\n{program_code}"""

    return basic_prompt

total_tokens = 0
total_files = 0
for file in folder_voting.glob("*.rs"):                                 # Cambiar folder de programs
    program_code = file.read_text(encoding="utf-8")
    
    tokens = tokenizer.encode(prompt_complex(program_code))             # Cambiar prompt a Basic o Complex
    
    print(f"{file.name}: {len(tokens)} tokens")
    total_tokens += len(tokens)
    total_files += 1

print(f"\nTotal tokens for all files: {total_tokens}")
print(f"\nTotal files: {total_files}")