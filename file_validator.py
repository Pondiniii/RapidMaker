import re


def file_check(file_content: bytes, file_name: str) -> tuple:
    # Sprawdzanie magic number dla plików STL (zaczyna się od "solid" dla ASCII albo 80-bajtowy nagłówek dla binarnych)
    if not (file_content.startswith(b'solid') or len(file_content) > 80):
        return False, "Invalid STL file"

    # Oddziel rozszerzenie od nazwy
    name, extension = file_name.rsplit('.', 1)

    # Sprawdzanie długości nazwy
    if len(name) > 60:
        name = name[:60]  # skrócenie nazwy

    # Usuwanie znaków specjalnych z nazwy i zastępowanie ich pustym łańcuchem
    sanitized_name = re.sub(r'[^a-zA-Z0-9]', '', name)

    # Połącz nazwę z rozszerzeniem
    final_name = f"{sanitized_name}.{extension}"

    return True, final_name


def test_file_check():
    with open("sample.stl", "rb") as file:
        content = file.read()
        result, sanitized_name = file_check(content, "sample.stl")

        if result:
            print(f"File is a valid STL file. Sanitized name: {sanitized_name}")
        else:
            print(f"File check failed. Reason: {sanitized_name}")

