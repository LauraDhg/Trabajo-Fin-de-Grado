# Creado por: Laura De Haro García
# Descripción: El siguiente código automatiza el envío de peticiones al modelo LLM DeepSeek V3.1 por medio de su API oficial. 
# Se hacen 63 peticiones en total para cada prompt (básico o complejo).
# Cada respuesta generada se guarda en una carpeta llamada "response", también se crea un archivo de texto donde se registra el tiempo que 
# se tardó en realizar cada petición y el tiempo total de todas las peticiones el cual se guarda en la carpeta "time-notes"

from openai import OpenAI
from pathlib import Path
import time

client = OpenAI(api_key="API-KEY", base_url="https://api.deepseek.com")
output_folder = Path("responses")
file_path = Path("time-notes")

# Envía la petición a la API y devuelve su respuesta
def send_request(file):

    # Program .rs pasado a texto plano
    program_code = file.read_text(encoding="utf-8")

    # CURRENT PROMPT: BASIC
    system_message = {
        "role": "system",
        "content": f"""You are a helpful assistant.
        Your output must strictly follow this format. 
        For every function you find in the code, 
        include its name and describe it using bullet points:

        Function "function_name":
        - Bullet point 1
        - Bullet point 2
        - ...

        Function "function_name":
        - Bullet point 1
        - Bullet point 2
        - ...

        Function "function_name":
        - Bullet point 1
        - Bullet point 2
        - ...

        Rules:
        - Analyse each function you find in the code.
        - Write your findings only as bullet points under each function.
        - Do not add text, explanations, or summaries outside this format.
        """
    }
    
    user_message = {
        "role": "user",
        "content": f"""Is this program correct? Search for 
        potential risks and vulnerabilities. 
        Program:\n\n{program_code}"""
    }

    # Petición a la API
    response = client.chat.completions.create(
        model="deepseek-chat",
        messages=[
            system_message,
            user_message,
        ],
        stream=False
    )

    return response

# Guarda la respuesta en un archivo de texto en la carpeta "response" con el nombre "num_response_lib.rs"
def process_response(response, cnt_program):

    text = response.choices[0].message.content
    text_name = output_folder / f"{cnt_program}_response_lib.txt"

    with open(text_name, "w", encoding="utf-8") as f:
        f.write(text)

    print(f"Saved: {text_name}")

def main():
    
    # Directorio de la carpeta con los contratos vulnerables WALLET
    program_path = Path("../../programs/vulnerable-programs-dataset/programs-AI")

    cnt_program = 1
    total_time = 0.0
    aux_path = program_path / f"{cnt_program}_lib.rs"
    text_name = file_path / f"Request_times.txt"

    output_folder.mkdir(parents=True, exist_ok=True)
    file_path.mkdir(parents=True, exist_ok=True)

    with open(text_name, "a", encoding="utf-8") as time_file:
        while aux_path.exists():

            start_time = time.perf_counter()
            # Petición
            response = send_request(aux_path)
            elapsed_time = time.perf_counter() - start_time
            time_file.write(f"Request {cnt_program} took: {elapsed_time:.2f} seconds\n")
            total_time += elapsed_time
            # Respuesta guardada
            process_response(response, cnt_program)
            cnt_program += 1
            aux_path = program_path / f"{cnt_program}_lib.rs"
        
        time_file.write(f"\nTotal time: {total_time:.2f} seconds\n")
        print(f"\nTotal time for all requests: {total_time:.2f} seconds")

if __name__ == "__main__":
    main()    

"""
PROMTP COMPLEJO:

    system_message = {
        "role": "system",
        "content": f"You are an expert developer of smart contracts on Solana using Rust, 
        with deep knowledge of the most advanced techniques available today. 
        Your task is to analyze the Rust code provided in the prompt for potential vulnerabilities.
        Identify functions that are deprecated, insecure, or pose potential risks, and provide clear explanations for each finding.
        Your output must strictly follow this format. For every function in the code, list its name and describe its characteristics 
        or vulnerabilities using bullet points:
        
        Function "function_name":
        - Bullet point 1
        - Bullet point 2
        - ...

        Function "function_name":
        - Bullet point 1
        - Bullet point 2
        - ...

        Function "function_name":
        - Bullet point 1
        - Bullet point 2
        - ...

        ...

        Rules:
        - Analyze each function you find in the code.
        - Use the exact function name inside quotes after the word Function.
        - Write only bullet points under each function.
        - Do not include explanations, summaries, or text outside this format.
        "
    }

    user_message = {
        "role": "user",
        "content": f"The following program is about to be deployed on Solana´s blockchain. 
        Please review it for any potential security vulnerabilities and suggest ways to mitigate them. 
        Program:\n\n{program_code}"
    }

"""