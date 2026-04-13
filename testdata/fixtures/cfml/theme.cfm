<cfset theme = "Dracula">

<cffunction name="renderTheme" access="public" returntype="string">
  <cfreturn theme>
</cffunction>

<cfoutput>#theme#</cfoutput>
