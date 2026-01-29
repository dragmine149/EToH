import pywikibot

for i in range(1000):
    site = pywikibot.Site(url="https://jtoh.fandom.com/api.php")
    pywikibot.Page(site, "ToAST").get(True, True)
