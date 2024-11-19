from django.urls import path
from . import views

urlpatterns = [
    path('phantom/callback', views.phantom_callback, name='phantom_callback'),
]
