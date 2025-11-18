# Creado por: Laura De Haro García
# Descripción: Este programa crea un set de contratos vulnerables para todas las posibles combinaciones de vulnerbaiildades presentes en Voting Program. 
# Por tanto, genera un set en total de 63 contratos.
# Crea las distintas combinaciones apartir del archivo lib-original.rs ubicado en contracts-original, este programa presenta todos los controles seguros.
# Crea varias carpteas:
# - contracts-dataset: contratos vulnerables con código de las vulnerabilidades presentes en el nombre del fichero
# - contracts-AI: contratos vulnerables sin ningún código en el nombre del fichero (son los que serán analizados por los LLMs)
# - contracts-dataset: contratos vulnerables con código de las vulnerabilidades presentes en el nombre del fichero

import os
import shutil
import itertools
import time

folder = "programs-dataset"
folder_base = "program-original"
folder_AI = "programs-AI"

vulns_detected = ["A_BUMP", "B_ADMIN_CHECK", "C_ACCOUNT_CONFUSION", "D_CPI", "E_OVERFLOW", "F_UNDERFLOW"]
vulns_dictionary = {"A_BUMP":"bump", "B_ADMIN_CHECK":"admin", "C_ACCOUNT_CONFUSION":"accountConfusion", "D_CPI":"cpi", "E_OVERFLOW":"overflow", "F_UNDERFLOW":"underflow"}

fun_create_admin = {"A_BUMP":{"BUMP_GOOD":(182,184), "BUMP_BAD":(184,186)}, "D_CPI":(191,195)}
fun_update_admin = {"A_BUMP":{"BUMP_GOOD":(251,253), "BUMP_BAD":(253,255)}, "B_ADMIN_CHECK":(260,264), "D_CPI":(265,269)}
fun_create_user = {"A_BUMP":{"BUMP_GOOD":(300,302), "BUMP_BAD":(302,304)}, "C_ACCOUNT_CONFUSION":(311,313)}
fun_proposal = {"A_BUMP":{"BUMP_GOOD":(372,374), "BUMP_BAD":(374,376)}, "C_ACCOUNT_CONFUSION":(381,388), "D_CPI":(389,393)}
fun_vote = {"A_BUMP":{"BUMP_GOOD":(467,469), "BUMP_BAD":(469,471)}, "C_ACCOUNT_CONFUSION":[(488,490),(481,485)], "D_CPI":(494,498), "E_OVERFLOW":[(515,516),(509,510)]}
fun_manage = {"A_BUMP":{"BUMP_GOOD":(571,573), "BUMP_BAD":(573,575)}, "B_ADMIN_CHECK":(580,587), "C_ACCOUNT_CONFUSION":(591,595), "E_OVERFLOW":(599,600), "F_UNDERFLOW":(602,603)}
fun_execute = {"A_BUMP":{"BUMP_GOOD":[(676,678),(664,666),(643,645)], "BUMP_BAD":[(678,680),(666,668),(645,647)]}, "B_ADMIN_CHECK":[(685,692),(652,659)], "D_CPI":(697,701)}
fun_make_decision = {"A_BUMP":{"BUMP_GOOD":(745,747), "BUMP_BAD":(747,749)}, "B_ADMIN_CHECK":(754,761), "D_CPI":(765,769)}

# Eliminar bump como parámetro si no existe la vulnerabilidad BUMPWRONG
list_bumps_reverse = [(728,729),(615,616),(553,554),(443,444),(351,352),(284,285),(233,234),(164,165)]

# Eliminar código si la vulnerabilidad ACCOUNT CONFUSION esta presente
last_lines_to_erase = [(51,52), (44,45),(31,35)] 

list_funs = [fun_make_decision, fun_execute, fun_manage, fun_vote, fun_proposal, fun_create_user, fun_update_admin, fun_create_admin]

# Crea las carpetas necesarias
def create_folder():
    if not os.path.exists(folder):
        os.makedirs(folder)

    if not os.path.exists(folder_AI):
        os.makedirs(folder_AI)

# Devuelve un set de frozen_sets, donde se alamacenan todas las posibles combinaciones entre vulnerabilidades
def combinations_vulnerabilities():

    result = set()
    for vuln in vulns_detected:
        result.add(frozenset([vuln]))
    
    for vuln_base in vulns_detected:
        resto = [v for v in vulns_detected if v != vuln_base]
        for r in range(1, len(resto) + 1):
            for combinacion in itertools.combinations(resto, r):
                comb = frozenset([vuln_base, *combinacion])
                result.add(comb)

    sorted_set = sorted(result, key=len)
    return sorted_set

# Modifica el contrato lib-original.rs sustituyendo por código inseguro o eliminando las líneas en las que aparecen los controles que evitan la vulnerabilidad en cuestión 
def change_contract(file_to_modify, set_vulns, list_funs, index, list_bumps):
    
    with open(file_to_modify, "r") as f:
        lineas = f.readlines()

    delete_lines = []
    replace_code = {} 
    
    for vuln in sorted(set_vulns): # NO FACE FALLTRA
        # Se elimina la vulnerabilidad presente en la función
        for func_aux in list_funs: 
            if vuln not in func_aux:
                continue
 
            # Si la vulnerabilidad Bump esta presente, se elimina la función segura
            if vuln == "A_BUMP":
                if isinstance(func_aux[vuln]["BUMP_BAD"], list):
                    for elem in func_aux[vuln]["BUMP_BAD"]:
                        ini, fin = elem
                        delete_lines.append((ini, fin))
                else:
                    ini,fin = func_aux[vuln]["BUMP_BAD"]
                    delete_lines.append((ini, fin))
           
            # Reemplazos de casos overflow y underflow por código inserguro        
            elif vuln == "E_OVERFLOW":
                if func_aux is fun_manage:
                    ini, fin = func_aux[vuln]
                    replace_code[(ini, fin)] = ["\tuser.amount_governance_points += amount;\n"]
                elif func_aux is fun_vote:
                    ini, fin = func_aux[vuln][0]
                    replace_code[(ini, fin)] = ["\tproposal.no_votes += user.amount_governance_points;\n"]
                    ini, fin = func_aux[vuln][1]
                    replace_code[(ini, fin)] = ["\tproposal.yes_votes += user.amount_governance_points;\n"]
            elif vuln == "F_UNDERFLOW":
                ini, fin = func_aux[vuln]
                replace_code[(ini, fin)] = ["\tuser.amount_governance_points -= amount;\n"]
            
            elif isinstance(func_aux[vuln], list):
                # Se eliminan vulnerabilidades que aparezcan repetidas en una función
                list_aux = func_aux[vuln]
                for elem in list_aux:
                    ini, fin = elem
                    delete_lines.append((ini, fin))
            else:
            # Se elimina la vulnerabilidad de la función
                ini, fin = func_aux[vuln]
                delete_lines.append((ini, fin))

    # Si la vulnerabilidad Bump no esta presente, se elimina la función insegura, los bumps pasados comoo parámetros y se sustituye la función process instruction
    if "A_BUMP" not in set_vulns: # es bueno
        for func_aux in list_funs:
            if isinstance(func_aux["A_BUMP"]["BUMP_GOOD"], list):
                for elem in func_aux["A_BUMP"]["BUMP_GOOD"]:
                    ini, fin = elem
                    delete_lines.append((ini, fin))
            else:
                ini,fin = func_aux["A_BUMP"]["BUMP_GOOD"]
                delete_lines.append((ini, fin))
            print(f"A_PDA_GOOD")

        for elem in list_bumps:
            ini, fin = elem
            delete_lines.append((ini, fin))

        # Process instruction seguro
        ini,fin = (143,160)
        text = """
                fn process_instruction(
                program_id: &Pubkey,
                accounts: &[AccountInfo],
                instruction_data: &[u8],
            ) -> ProgramResult {
                let instruction = InstructionFormat::unpack(instruction_data)?;
                match instruction {
                    InstructionFormat::CreateAdmin   {} => create_admin(program_id, accounts),
                    InstructionFormat::UpdateAdmin   {new_admin} => update_admin(program_id, accounts, new_admin),
                    InstructionFormat::CreateAccount {} => create_account(program_id, accounts),
                    InstructionFormat::OpenProposal  {title, description} => open_proposal(program_id, accounts, title, description),
                    InstructionFormat::Vote          {vote_answer} => vote_process(program_id, accounts, vote_answer),
                    InstructionFormat::ManagePoints   {amount, instruction} => manage_governance_points(program_id, accounts, amount,instruction),
                    InstructionFormat::ExecuteIntrs  {variant_instr, amount, vault_bump} => execute_instruction(program_id, accounts, variant_instr, amount, vault_bump),
                    InstructionFormat::MakeDecision  {} => make_decision(program_id, accounts),
                }
            }
        """
        replace_code[(ini, fin)] = text

    # Si la vulnerabilidad Account confusion esta presente, se elimina el código seguro
    if "C_ACCOUNT_CONFUSION" in set_vulns:
        for elem in last_lines_to_erase:
            ini, fin = elem
            delete_lines.append((ini, fin))

    apply_changes = list(replace_code.items()) + [(r, None) for r in delete_lines]

    apply_changes.sort(key=lambda x: x[0][0], reverse=True)

    for (ini, fin), new_text in apply_changes:
        if new_text is None:
            del lineas[ini:fin]
        else:
            lineas[ini:fin] = new_text

    # Guardar el archivo modificado
    with open(file_to_modify, "w") as f:
        f.writelines(lineas)

# Itera sobre el set con las combinaciones de todas las vulnerabilidades, y ejecuta la funcion para crear el contrato vulnerable correspondiente
def create_contracts(combinations):
    index_cnt = 0
    original_file = os.path.join(folder_base, "lib.rs")

    for set_vulns in combinations:
        index_cnt += 1

        # Nombre del contrato
        list_vulns = list(sorted(set_vulns))
        name = str(index_cnt) + "_lib"
        for vuln_aux in list_vulns:
            name += "_" + vulns_dictionary[vuln_aux]
        name +=".rs"

        # Nombre del contrato para los modelos LLM
        name_AI = str(index_cnt) + "_lib.rs"
 
        # Copia de lib-original.rs
        dest_directory = os.path.join(folder, name)
        copy_of_file = shutil.copyfile(original_file, dest_directory)

        # Modificación del contrato 
        change_contract(copy_of_file, set_vulns, list_funs, index_cnt, list_bumps_reverse)
        
        # Copia del fichero modificado para los modelos LLM
        dest_directory_AI = os.path.join(folder_AI, name_AI)
        copy_file_AI = shutil.copyfile(dest_directory, dest_directory_AI)

def main():
    comb = combinations_vulnerabilities()
    create_folder()
    create_contracts(comb)
   
if __name__ == "__main__":
    main()    


