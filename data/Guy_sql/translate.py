import sqlite3
import json

def row_to_json(row) -> tuple[str, dict]:
    return row[1], dict(difficulty=row[2], badge_id=row[0])

if __name__ == '__main__':
    data = [];

    with sqlite3.connect('etoh.sqlite3') as conn:
        cursor = conn.execute("SELECT tb.badge_id, t.name, t.difficulty FROM towers t JOIN tower_badges tb ON t.name = tb.name WHERE t.found_in = (SELECT a.name FROM areas a WHERE a.acronym = 'Z10') ORDER BY t.difficulty;")
        data = cursor.fetchall()
        cursor.close()

    # data = map(row_to_json, data);
    # print(data)

    # for row in data:
    #     print(row_to_json(row))

    # a = json.dumps(data);
    # print(a);

# ... # stuff to call row_to_json() for the rest
