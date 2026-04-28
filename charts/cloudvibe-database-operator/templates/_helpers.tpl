{{- define "cloudvibe-database-operator.name" -}}
{{- .Chart.Name -}}
{{- end -}}

{{- define "cloudvibe-database-operator.labels" -}}
app.kubernetes.io/name: {{ include "cloudvibe-database-operator.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}
