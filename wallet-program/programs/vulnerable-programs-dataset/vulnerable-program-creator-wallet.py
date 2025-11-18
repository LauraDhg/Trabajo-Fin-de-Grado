# Creado por: Laura De Haro García
# Descripción: Este programa crea un set de contratos vulnerables para todas las posibles combinaciones de vulnerbaiildades presentes en Wallet Program. 
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

vulns_detected = ["A_OWNCHECK", "B_SIGCHECK", "C_PDACHECK", "D_ACCOUNTINIT", "E_OVERFLOW", "F_UNDERFLOW"]
vulns_dictionary = {"A_OWNCHECK":"own", "B_SIGCHECK":"sig", "C_PDACHECK":"pda", "D_ACCOUNTINIT":"init", "E_OVERFLOW":"OvFlow", "F_UNDERFLOW":"UndFlow"}

fun_init     = {"A_OWNCHECK":(62,65),   "B_SIGCHECK":(66,69),   "C_PDACHECK":(71,74),   "D_ACCOUNTINIT":(75,79)}
fun_deposit  = {"A_OWNCHECK":(124,127), "B_SIGCHECK":(128,131), "C_PDACHECK":(132,136), "D_ACCOUNTINIT":(141,144),"E_OVERFLOW":(147,148)}
fun_withdraw = {"A_OWNCHECK":(168,171), "B_SIGCHECK":(172,175), "C_PDACHECK":(176,180), "D_ACCOUNTINIT":(185,188), "F_UNDERFLOW":(191,196)}
fun_transfer = {"A_OWNCHECK":(220,223), "B_SIGCHECK":(224,227), "C_PDACHECK":(238,233), "D_ACCOUNTINIT":(241,247),"E_OVERFLOW":(251,252), "F_UNDERFLOW":(253,258)}

list_funs = [fun_transfer, fun_withdraw, fun_deposit, fun_init]

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
def change_contract(file_to_modify, set_vulns, list_funs, index):
    
    with open(file_to_modify, "r") as f:
        lineas = f.readlines()

    delete_lines = []
    replace_code = {}

    for vuln in sorted(set_vulns): # NO HACE FALTA sorted(set_vulns, reverse=True
     # Se elimina la vulnerabilidad presente en la función
     for func_aux in list_funs:
        if vuln not in func_aux:
            continue 

        ini, fin = func_aux[vuln]

        # Casos especiales con reemplazo
        if vuln == "E_OVERFLOW":
            if func_aux is fun_transfer:
                replace_code[(ini, fin)] = ["\tdest_wallet.amount += amount;\n"]
            else:
                replace_code[(ini, fin)] = ["\twallet.amount += amount;\n"]
        elif vuln == "F_UNDERFLOW":
            if func_aux is fun_transfer:
                replace_code[(ini, fin)] = ["\tsrc_wallet.amount -= amount;\n"]
            else:
                replace_code[(ini, fin)] = ["\twallet.amount -= amount;\n"]
        else:
            # Se elimina la vulnerabilidad de la función
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
    original_file = os.path.join(folder_base, "lib-original.rs")

    for set_vulns in combinations:
        index_cnt += 1
        list_vulns = list(sorted(set_vulns))

        # Nombre del contrato
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
        change_contract(copy_of_file, set_vulns, list_funs, index_cnt)
        
        # Copia del fichero modificado para los modelos LLM
        dest_directory_AI = os.path.join(folder_AI, name_AI)
        copy_file_AI = shutil.copyfile(dest_directory, dest_directory_AI)

def main():
    comb = combinations_vulnerabilities()
    create_folder()
    create_contracts(comb)
   
if __name__ == "__main__":
    main()    


