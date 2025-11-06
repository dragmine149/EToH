import pywikibot as pyb

site = pyb.Site('en', 'etoh')
page = pyb.Page(site, 'Zone 8') # badge name
# pass
text = page.get()
import wikitextparser as wtp

content = wtp.parse(text)
templates = content.templates
templates_len = len(templates)
for template in templates:
	if "infobox" in template.name:
		return this

import pywikibot

# import pywikibot.pagegenerators
import wikitextparser
from pywikibot import pagegenerators

site = pywikibot.Site(url="https://jtoh.fandom.com/api.php")
to_search = "Error 404: Unable to detect the tower"
for page in pagegenerators.SearchPageGenerator(to_search, None, None, site):
    p = wikitextparser.parse(page.get())
    for link in p.external_links:
        if link.text is not None and to_search in link.text:
            print("Found", page)
            break
